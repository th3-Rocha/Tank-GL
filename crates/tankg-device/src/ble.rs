//! Bluetooth LE transport through BlueZ (D-Bus).
//!
//! `TANK-Gv2_BLE` has its own `ae40` service: the raw frame (no SysEx, no
//! 7-bit packing) goes to `ae41` (write-without-response) and the reply comes
//! back as a notification on `ae42`. Frames bigger than one packet can be
//! split: the pedal joins them.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use dbus::arg::{PropMap, RefArg, Variant};
use dbus::blocking::stdintf::org_freedesktop_dbus::{
    ObjectManager, Properties, PropertiesPropertiesChanged,
};
use dbus::blocking::{Connection, Proxy};
use dbus::message::MatchRule;
use dbus::{Path, arg};
use tankg_proto::Frame;

use crate::{Error, Result};

const BLUEZ: &str = "org.bluez";
const DEVICE: &str = "org.bluez.Device1";
const CHAR: &str = "org.bluez.GattCharacteristic1";
const TX: &str = "0000ae41-0000-1000-8000-00805f9b34fb";
const RX: &str = "0000ae42-0000-1000-8000-00805f9b34fb";
const CALL: Duration = Duration::from_secs(5);
const CONNECT: Duration = Duration::from_secs(15);

fn bt(e: impl std::fmt::Display) -> Error {
    Error::Bluetooth(e.to_string())
}

type Objects = HashMap<Path<'static>, HashMap<String, PropMap>>;

fn objects(conn: &Connection) -> Result<Objects> {
    conn.with_proxy(BLUEZ, "/", CALL)
        .get_managed_objects()
        .map_err(bt)
}

fn prop_str<'a>(props: &'a PropMap, key: &str) -> Option<&'a str> {
    props.get(key)?.0.as_str()
}

fn prop_bool(props: &PropMap, key: &str) -> bool {
    props
        .get(key)
        .and_then(|v| v.0.as_u64())
        .is_some_and(|v| v != 0)
}

/// Control is `TANK-Gv2_BLE`; `TANK-Gv2` without the suffix is audio only.
fn find_device(objs: &Objects) -> Option<(Path<'static>, String, bool)> {
    objs.iter().find_map(|(path, ifs)| {
        let dev = ifs.get(DEVICE)?;
        let name = prop_str(dev, "Name").or_else(|| prop_str(dev, "Alias"))?;
        (name.starts_with("TANK-G") && name.ends_with("_BLE")).then(|| {
            (
                path.clone(),
                name.to_string(),
                prop_bool(dev, "Connected") && prop_bool(dev, "ServicesResolved"),
            )
        })
    })
}

fn find_char(objs: &Objects, dev: &Path, uuid: &str) -> Option<Path<'static>> {
    let prefix = format!("{dev}/");
    objs.iter().find_map(|(path, ifs)| {
        let c = ifs.get(CHAR)?;
        (path.starts_with(&prefix) && prop_str(c, "UUID") == Some(uuid)).then(|| path.clone())
    })
}

pub(crate) struct Ble {
    conn: Connection,
    tx: Path<'static>,
    /// Biggest piece that fits one write-without-response (MTU − 3).
    chunk: usize,
    stop: Arc<AtomicBool>,
    listener: Option<JoinHandle<()>>,
    pub(crate) name: String,
}

impl Ble {
    /// Opens the pedal over Bluetooth. With `connect`, asks BlueZ to connect
    /// if it is paired but disconnected; without it, only uses an existing
    /// connection (BlueZ reconnects trusted devices on its own).
    pub(crate) fn open(frames: Sender<Frame>, connect: bool) -> Result<Self> {
        let conn = Connection::new_system().map_err(bt)?;
        let (dev, name, ready) = find_device(&objects(&conn)?).ok_or(Error::NotFound)?;
        if !ready {
            if !connect {
                return Err(Error::NotFound);
            }
            let proxy = conn.with_proxy(BLUEZ, &dev, CONNECT);
            proxy
                .method_call::<(), _, _, _>(DEVICE, "Connect", ())
                .map_err(bt)?;
            let deadline = Instant::now() + CONNECT;
            while !proxy
                .get::<bool>(DEVICE, "ServicesResolved")
                .unwrap_or(false)
            {
                if Instant::now() > deadline {
                    return Err(Error::Bluetooth(
                        "connected, but the services did not show up".into(),
                    ));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }

        let objs = objects(&conn)?;
        let missing = || Error::Bluetooth(format!("{name} has no ae40 service"));
        let tx = find_char(&objs, &dev, TX).ok_or_else(missing)?;
        let rx = find_char(&objs, &dev, RX).ok_or_else(missing)?;
        let mtu: u16 = conn
            .with_proxy(BLUEZ, &tx, CALL)
            .get(CHAR, "MTU")
            .unwrap_or(23);
        let chunk = (mtu as usize).saturating_sub(3).max(20);

        let stop = Arc::new(AtomicBool::new(false));
        let listener = Self::listen(rx, frames, stop.clone())?;
        Ok(Self {
            conn,
            tx,
            chunk,
            stop,
            listener: Some(listener),
            name,
        })
    }

    /// Thread with its own D-Bus connection: enables `ae42` notifications,
    /// joins the pieces and delivers whole frames. Returns only once
    /// notifications are on, so the first reply isn't lost.
    fn listen(
        rx: Path<'static>,
        frames: Sender<Frame>,
        stop: Arc<AtomicBool>,
    ) -> Result<JoinHandle<()>> {
        let (ready_tx, ready_rx) = mpsc::channel::<Result<()>>();
        let handle = std::thread::Builder::new()
            .name("tankg-ble".into())
            .spawn(move || {
                let setup = || -> Result<Connection> {
                    let conn = Connection::new_system().map_err(bt)?;
                    let mut buf = Vec::new();
                    let rule = MatchRule::new_signal(
                        "org.freedesktop.DBus.Properties",
                        "PropertiesChanged",
                    )
                    .with_path(rx.clone());
                    conn.add_match(rule, move |p: PropertiesPropertiesChanged, _, _| {
                        if let Some(v) = p
                            .changed_properties
                            .get("Value")
                            .and_then(|v| arg::cast::<Vec<u8>>(&v.0))
                        {
                            buf.extend_from_slice(v);
                            for f in drain_frames(&mut buf) {
                                let _ = frames.send(f);
                            }
                        }
                        true
                    })
                    .map_err(bt)?;
                    conn.with_proxy(BLUEZ, &rx, CALL)
                        .method_call::<(), _, _, _>(CHAR, "StartNotify", ())
                        .map_err(bt)?;
                    Ok(conn)
                };
                let conn = match setup() {
                    Ok(c) => {
                        let _ = ready_tx.send(Ok(()));
                        c
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                while !stop.load(Ordering::Relaxed) {
                    if conn.process(Duration::from_millis(100)).is_err() {
                        break;
                    }
                }
                let _ = conn
                    .with_proxy(BLUEZ, &rx, CALL)
                    .method_call::<(), _, _, _>(CHAR, "StopNotify", ());
            })
            .map_err(bt)?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(handle),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(Error::Bluetooth("the Bluetooth listener stopped".into())),
        }
    }

    pub(crate) fn send(&self, frame: &Frame) -> Result<()> {
        let proxy: Proxy<&Connection> = self.conn.with_proxy(BLUEZ, &self.tx, CALL);
        for part in frame.to_raw().chunks(self.chunk) {
            let mut opts: PropMap = HashMap::new();
            opts.insert(
                "type".into(),
                Variant(Box::new("command".to_string()) as Box<dyn RefArg>),
            );
            proxy
                .method_call::<(), _, _, _>(CHAR, "WriteValue", (part.to_vec(), opts))
                .map_err(bt)?;
        }
        Ok(())
    }
}

impl Drop for Ble {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.listener.take() {
            let _ = h.join();
        }
    }
}

/// Takes the complete frames out of `buf`. Junk before `00 59` is dropped.
fn drain_frames(buf: &mut Vec<u8>) -> Vec<Frame> {
    let mut out = Vec::new();
    loop {
        match buf.windows(2).position(|w| w == tankg_proto::MAGIC) {
            Some(i) => drop(buf.drain(..i)),
            None => {
                // Keep a lone 00 at the end: it may start the next frame.
                let keep = usize::from(buf.last() == Some(&0));
                buf.drain(..buf.len() - keep);
                return out;
            }
        }
        if buf.len() < 6 {
            return out;
        }
        let total = 6 + u32::from_le_bytes([buf[3], buf[4], buf[5], 0]) as usize + 1;
        if buf.len() < total {
            return out;
        }
        if let Ok(f) = Frame::from_raw(&buf[..total]) {
            out.push(f);
            buf.drain(..total);
        } else {
            // Bad checksum: skip this 00 59 and look for the next one.
            buf.drain(..2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_pieces_and_drops_junk() {
        let a = Frame::new(0x23, vec![1, 2, 3]).to_raw();
        let b = Frame::new(0x00, vec![0]).to_raw();
        let mut wire = vec![0xAA, 0xBB];
        wire.extend(&a);
        wire.extend(&b);
        let mut buf = Vec::new();
        let mut got = Vec::new();
        for part in wire.chunks(4) {
            buf.extend_from_slice(part);
            got.extend(drain_frames(&mut buf));
        }
        assert_eq!(
            got,
            vec![Frame::new(0x23, vec![1, 2, 3]), Frame::new(0x00, vec![0])]
        );
        assert!(buf.is_empty());
    }
}
