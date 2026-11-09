//! File dialogs off the UI thread.
//!
//! `rfd`'s blocking dialogs stall the window's event loop while they are open,
//! so the compositor's pings go unanswered and Hyprland reports the app as
//! "not responding". The dialog runs on its own thread and the result is
//! picked up on a later frame.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};

#[derive(Default)]
pub struct PendingDialog(Option<Receiver<Option<PathBuf>>>);

impl PendingDialog {
    /// Opens `dialog` on another thread. Ignored while one is already open.
    pub fn start(
        &mut self,
        ctx: &egui::Context,
        dialog: impl FnOnce() -> Option<PathBuf> + Send + 'static,
    ) {
        if self.is_open() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(dialog());
            ctx.request_repaint();
        });
        self.0 = Some(rx);
    }

    pub fn is_open(&self) -> bool {
        self.0.is_some()
    }

    /// The chosen path, once, when the dialog closes with a choice.
    pub fn take(&mut self) -> Option<PathBuf> {
        let rx = self.0.as_ref()?;
        match rx.try_recv() {
            Ok(path) => {
                self.0 = None;
                path
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.0 = None;
                None
            }
        }
    }
}
