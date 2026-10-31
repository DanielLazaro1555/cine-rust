// Persistent stdout/stderr capture for the native application and libmpv.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;

/// Redirect application output to a persistent per-user log before GTK/libmpv
/// start. Keep one rotated log so diagnostics survive the next launch.
pub fn init() {
    match log_path() {
        Ok(path) => {
            if let Err(error) = redirect_output(&path) {
                eprintln!("[CineRust] No se pudo activar el archivo de logs {}: {error}", path.display());
                return;
            }
            eprintln!(
                "\n[CineRust] Inicio de sesión de logs: unix={} | archivo={}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|duration| duration.as_secs())
                    .unwrap_or_default(),
                path.display()
            );
        }
        Err(error) => eprintln!("[CineRust] No se pudo preparar el archivo de logs: {error}"),
    }
}

fn log_path() -> std::io::Result<PathBuf> {
    let root = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or(std::env::current_dir()?.join(".local-state"));
    let directory = root.join("cine");
    fs::create_dir_all(&directory)?;
    Ok(directory.join("cine.log"))
}

fn redirect_output(path: &PathBuf) -> std::io::Result<()> {
    if fs::metadata(path).is_ok_and(|metadata| metadata.len() >= MAX_LOG_BYTES) {
        let previous = path.with_extension("log.1");
        if previous.exists() {
            fs::remove_file(&previous)?;
        }
        fs::rename(path, previous)?;
    }

    let log = OpenOptions::new().create(true).append(true).open(path)?;
    // Save a duplicate of the file descriptor before `log` is dropped. stdout
    // and stderr then share the append-only sink, including mpv and panic output.
    let fd = log.as_raw_fd();
    for target in [libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        // SAFETY: `fd` is an open file descriptor and `target` is a standard
        // process descriptor. dup2 gives the target its own reference.
        if unsafe { libc::dup2(fd, target) } == -1 {
            return Err(std::io::Error::last_os_error());
        }
    }
    drop(log);
    Ok(())
}
