//! Tank Studio: shell Tauri sobre `tankg-device`.
//!
//! O app é o único dono da porta MIDI enquanto está aberto. Qualquer erro de
//! transporte derruba a conexão, e a próxima chamada da interface reabre.

use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;
use tankg_device::TankG;
use tankg_proto::{Mode, Patch, PresetId, Status};

#[derive(Default)]
struct Device(Mutex<Option<TankG>>);

impl Device {
    fn with<T>(&self, f: impl FnOnce(&mut TankG) -> tankg_device::Result<T>) -> Result<T, String> {
        let mut guard = self.0.lock().map_err(|_| "estado do pedal corrompido".to_string())?;
        if guard.is_none() {
            *guard = Some(TankG::open().map_err(|e| e.to_string())?);
        }
        let r = f(guard.as_mut().unwrap());
        if r.is_err() {
            *guard = None;
        }
        r.map_err(|e| e.to_string())
    }
}

#[derive(Serialize)]
struct DeviceInfo {
    firmware: String,
    port: String,
}

#[derive(Serialize)]
struct DeviceStatus {
    preset: Option<String>,
    mode: &'static str,
}

impl From<Status> for DeviceStatus {
    fn from(st: Status) -> Self {
        Self {
            preset: st.preset.map(|p| p.to_string()),
            mode: match st.mode {
                Mode::Preset => "P",
                Mode::Live => "L",
            },
        }
    }
}

#[tauri::command]
fn device_info(dev: tauri::State<Device>) -> Result<DeviceInfo, String> {
    dev.with(|d| Ok(DeviceInfo { firmware: d.identify()?, port: d.port_name().to_string() }))
}

#[tauri::command]
fn device_status(dev: tauri::State<Device>) -> Result<DeviceStatus, String> {
    dev.with(|d| d.status()).map(Into::into)
}

#[derive(Serialize)]
struct DeviceState {
    status: DeviceStatus,
    /// Preset atual (edit buffer). No modo LIVE acompanha os knobs na hora.
    patch: Patch,
}

#[tauri::command]
fn device_state(dev: tauri::State<Device>) -> Result<DeviceState, String> {
    dev.with(|d| {
        let status = d.status()?;
        let raw = d.read_current_patch()?;
        Ok((status, raw))
    })
    .and_then(|(status, raw)| {
        let patch = Patch::parse(&raw).ok_or("preset atual com tamanho errado")?;
        Ok(DeviceState { status: status.into(), patch })
    })
}

#[tauri::command]
fn select_preset(dev: tauri::State<Device>, id: String) -> Result<DeviceStatus, String> {
    let p: PresetId = id.parse()?;
    dev.with(|d| {
        d.select_preset(p)?;
        // O pedal leva um instante para atualizar o estado.
        sleep(Duration::from_millis(150));
        d.status()
    })
    .map(Into::into)
}

#[tauri::command]
fn backup(dev: tauri::State<Device>, path: String) -> Result<usize, String> {
    let bank = dev.with(|d| d.read_preset_bank())?;
    if bank[..2] != [0xA5, 0x5A] {
        return Err(format!("banco lido não começa com a5 5a: {:02x?}", &bank[..8]));
    }
    std::fs::write(&path, &bank).map_err(|e| format!("gravando {path}: {e}"))?;
    Ok(bank.len())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Device::default())
        .invoke_handler(tauri::generate_handler![device_info, device_status, device_state, select_preset, backup])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Tank Studio");
}
