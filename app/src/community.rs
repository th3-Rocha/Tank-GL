//! community.m-vave.com client: search and download amps (.am4data) and IRs
//! (.wav). The listing and the files are public; no account needed.
//!
//! Endpoints taken from the site itself (`/static/js/index-*.js`):
//! `POST /bbs/api/getlist` e `GET /bbs/api/audition/model-file?name=…`.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use serde::Deserialize;

const BASE: &str = "https://community.m-vave.com/bbs/api";
pub const SITE: &str = "https://community.m-vave.com/";
const PAGE_SIZE: u32 = 40;
/// No file useful to the Tank-G is bigger than this (a long WAV IR is ~1 MB).
const MAX_FILE: u64 = 4 * 1024 * 1024;

/// File type the Tank-G uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileKind {
    Amp,
    Ir,
}

impl FileKind {
    fn api_type(self) -> &'static str {
        match self {
            FileKind::Amp => "am4data",
            FileKind::Ir => "wav",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub title: String,
    pub author: String,
    pub downloads: u64,
    pub files: Vec<RemoteFile>,
}

#[derive(Clone, Debug)]
pub struct RemoteFile {
    /// Name on the server (`uuid.am4Data`).
    pub file_name: String,
    /// Name to display.
    pub label: String,
    pub size: String,
}

/// What the listing alone says about compatibility (before downloading).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fit {
    /// Amp up to 8 KB or short IR: should go straight in.
    Good,
    /// Medium IR: the tail may be cut.
    Maybe,
    /// Long IR (will be cut) or an amp for another pedal (doesn't fit).
    Bad,
    Unknown,
}

impl RemoteFile {
    fn kb(&self) -> Option<f32> {
        let t = self.size.trim().to_ascii_uppercase();
        if let Some(n) = t.strip_suffix("KB") {
            n.trim().parse().ok()
        } else if let Some(n) = t.strip_suffix("MB") {
            n.trim().parse::<f32>().ok().map(|v| v * 1024.0)
        } else {
            None
        }
    }

    /// The Tank-G takes amps up to 8 KiB and IRs of 2048 samples (~6 KB at 24-bit mono).
    pub fn fit(&self, kind: FileKind) -> Fit {
        let Some(kb) = self.kb() else {
            return Fit::Unknown;
        };
        match kind {
            FileKind::Amp if kb <= 8.0 => Fit::Good,
            FileKind::Amp => Fit::Bad,
            FileKind::Ir if kb <= 13.0 => Fit::Good,
            FileKind::Ir if kb <= 40.0 => Fit::Maybe,
            FileKind::Ir => Fit::Bad,
        }
    }
}

/// Replies carry the requested type: if the tab changed meanwhile, the UI drops them.
pub enum Reply {
    Results {
        kind: FileKind,
        query: String,
        items: Vec<Item>,
        total: u64,
    },
    File {
        kind: FileKind,
        label: String,
        bytes: Vec<u8>,
    },
    Error(String),
}

enum Req {
    Search { query: String, kind: FileKind },
    Download(FileKind, RemoteFile),
}

pub struct Community {
    tx: Sender<Req>,
    pub rx: Receiver<Reply>,
    pub busy: bool,
}

impl Community {
    pub fn spawn(ctx: egui::Context) -> Self {
        let (tx, req_rx) = mpsc::channel::<Req>();
        let (reply_tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("community".into())
            .spawn(move || {
                let agent = ureq::AgentBuilder::new()
                    .timeout(Duration::from_secs(20))
                    .user_agent(concat!("tank-studio/", env!("CARGO_PKG_VERSION")))
                    .build();
                for req in req_rx {
                    let reply = match req {
                        Req::Search { query, kind } => search(&agent, &query, kind)
                            .map(|(items, total)| Reply::Results {
                                kind,
                                query,
                                items,
                                total,
                            })
                            .unwrap_or_else(Reply::Error),
                        Req::Download(kind, f) => download(&agent, &f.file_name)
                            .map(|bytes| Reply::File {
                                kind,
                                label: f.label.clone(),
                                bytes,
                            })
                            .unwrap_or_else(Reply::Error),
                    };
                    let _ = reply_tx.send(reply);
                    ctx.request_repaint();
                }
            })
            .expect("community thread");
        Self {
            tx,
            rx,
            busy: false,
        }
    }

    pub fn search(&mut self, query: String, kind: FileKind) {
        self.busy = self.tx.send(Req::Search { query, kind }).is_ok();
    }

    pub fn download(&mut self, kind: FileKind, f: RemoteFile) {
        self.busy = self.tx.send(Req::Download(kind, f)).is_ok();
    }
}

#[derive(Deserialize)]
struct Resp<T> {
    code: i64,
    msg: Option<String>,
    data: Option<T>,
}

#[derive(Deserialize)]
struct Page {
    records: Vec<Record>,
    total: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    title: String,
    file_path: Option<String>,
    file_type: Option<String>,
    create_user_name: Option<String>,
    download_num: Option<u64>,
    is_deleted: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileEntry {
    file_name: String,
    original_name: Option<String>,
    file_size: Option<String>,
}

fn search(agent: &ureq::Agent, query: &str, kind: FileKind) -> Result<(Vec<Item>, u64), String> {
    let body = serde_json::json!({
        "search": query.trim(),
        "select": [],
        "sort": 0,
        "currentPage": 1,
        "pageSize": PAGE_SIZE,
    });
    let resp: Resp<Page> = agent
        .post(&format!("{BASE}/getlist"))
        .send_json(body)
        .map_err(|e| format!("community: {e}"))?
        .into_json()
        .map_err(|e| format!("community: unexpected reply ({e})"))?;
    if resp.code != 200 {
        return Err(format!(
            "community: {}",
            resp.msg.unwrap_or_else(|| format!("code {}", resp.code))
        ));
    }
    let page = resp.data.ok_or("community: reply without data")?;
    Ok((
        page.records
            .into_iter()
            .filter_map(|r| item(r, kind))
            .collect(),
        page.total,
    ))
}

/// Only posts of the right type and only files with the right extension.
fn item(r: Record, kind: FileKind) -> Option<Item> {
    if r.is_deleted.unwrap_or(0) != 0
        || !r
            .file_type
            .as_deref()?
            .eq_ignore_ascii_case(kind.api_type())
    {
        return None;
    }
    let entries: Vec<FileEntry> = serde_json::from_str(r.file_path.as_deref()?).ok()?;
    let files: Vec<RemoteFile> = entries
        .into_iter()
        .filter(|f| {
            f.file_name
                .to_ascii_lowercase()
                .ends_with(&format!(".{}", kind.api_type()))
        })
        .map(|f| RemoteFile {
            label: label(f.original_name.as_deref(), &f.file_name),
            size: f.file_size.unwrap_or_default(),
            file_name: f.file_name,
        })
        .collect();
    (!files.is_empty()).then(|| Item {
        title: r.title,
        author: r.create_user_name.unwrap_or_default(),
        downloads: r.download_num.unwrap_or(0),
        files,
    })
}

/// As the site shows it: original name (without extension) + the real file's
/// extension. An amp's original is usually the WAV/NAM used for training.
fn label(original: Option<&str>, file_name: &str) -> String {
    let Some(orig) = original.filter(|o| !o.is_empty()) else {
        return display_name(file_name);
    };
    let stem = orig.rsplit_once('.').map_or(orig, |(s, _)| s);
    match file_name.rsplit_once('.') {
        Some((_, ext)) => format!("{stem}.{ext}"),
        None => orig.to_owned(),
    }
}

/// `uuid_Name.am4Data` → `Name.am4Data`; a bare uuid stays as is.
fn display_name(file_name: &str) -> String {
    match file_name.split_once('_') {
        Some((head, rest)) if head.len() == 36 && head.matches('-').count() == 4 => rest.to_owned(),
        _ => file_name.to_owned(),
    }
}

fn download(agent: &ureq::Agent, file_name: &str) -> Result<Vec<u8>, String> {
    let resp = agent
        .get(&format!("{BASE}/audition/model-file"))
        .query("name", file_name)
        .call()
        .map_err(|e| format!("download: {e}"))?;
    let mut bytes = Vec::new();
    use std::io::Read;
    resp.into_reader()
        .take(MAX_FILE + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("download: {e}"))?;
    if bytes.len() as u64 > MAX_FILE {
        return Err("download: file too big for the Tank-G".into());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_by_kind_and_extension() {
        let r = Record {
            title: "Pack".into(),
            file_path: Some(r#"[{"fileName":"4409ba29-93f3-42ec-b6f2-7bf100b35e88_1CT-OD.am4Data","fileSize":"8KB"},{"fileName":"x.am3Data"}]"#.into()),
            file_type: Some("am4data".into()),
            create_user_name: Some("Long Live".into()),
            download_num: Some(3),
            is_deleted: Some(0),
        };
        let it = item(r, FileKind::Amp).unwrap();
        assert_eq!(it.files.len(), 1);
        assert_eq!(it.files[0].label, "1CT-OD.am4Data");
        let wav = Record {
            title: "IR".into(),
            file_path: Some(r#"[{"fileName":"a.wav"}]"#.into()),
            file_type: Some("wav".into()),
            create_user_name: None,
            download_num: None,
            is_deleted: None,
        };
        assert!(item(wav, FileKind::Amp).is_none());
        assert_eq!(
            label(Some("[AMP] Plexi - DI.nam"), "5409a594.am4Data"),
            "[AMP] Plexi - DI.am4Data"
        );
        let f = |size: &str| RemoteFile {
            file_name: String::new(),
            label: String::new(),
            size: size.into(),
        };
        assert_eq!(f("8KB").fit(FileKind::Amp), Fit::Good);
        assert_eq!(f("14KB").fit(FileKind::Amp), Fit::Bad);
        assert_eq!(f("6KB").fit(FileKind::Ir), Fit::Good);
        assert_eq!(f("153KB").fit(FileKind::Ir), Fit::Bad);
    }
}
