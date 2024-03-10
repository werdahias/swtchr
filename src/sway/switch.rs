use std::sync::{Mutex, OnceLock};

use eyre::WrapErr;
use swayipc::Connection;

use super::subscribe::SwayWindowId;

fn connection() -> &'static Mutex<Connection> {
    static CONNECTION: OnceLock<Mutex<Connection>> = OnceLock::new();

    CONNECTION.get_or_init(|| {
        Mutex::new(Connection::new().expect("failed acquiring a Sway IPC connection"))
    })
}

pub fn switch_window(id: SwayWindowId) -> eyre::Result<()> {
    connection()
        .lock()
        .expect("lock is poisoned")
        .run_command(format!("[con_id=\"{}\"] focus", id.0))
        .wrap_err("failed running Sway window switch command")?
        .into_iter()
        .collect::<Result<_, _>>()
        .wrap_err("failed running Sway window switch command")
}