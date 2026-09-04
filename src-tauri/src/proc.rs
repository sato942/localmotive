//! Child-process construction that never flashes a console window.
//!
//! Localmotive is a windowed application, so it owns no console. Every
//! `std::process::Command` it spawns on Windows would therefore allocate a
//! fresh console window: `nvidia-smi`, the PowerShell adapter query, the
//! `llama-server --version` / `--help` capability probes, and the served
//! model itself. Those windows appear and disappear as visible flashes.
//!
//! All child processes must be created through [`hidden_command`], which sets
//! `CREATE_NO_WINDOW` on Windows and behaves like a plain `Command` elsewhere.

use std::ffi::OsStr;
use std::process::Command;

/// `CREATE_NO_WINDOW`: run the child without allocating a console window.
#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Build a `Command` that does not create a console window.
pub fn hidden_command<S: AsRef<OsStr>>(program: S) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    /// Source-level invariant: no module may bypass `hidden_command`, because a
    /// single direct `Command::new` reintroduces the console flash.
    #[test]
    fn no_module_constructs_a_raw_command() {
        let sources = [
            "core.rs",
            "runtime.rs",
            "lib.rs",
            "main.rs",
            "gguf.rs",
            "tune.rs",
            "cloud.rs",
        ];
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for name in sources {
            let path = dir.join(name);
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            assert!(
                !text.contains("Command::new("),
                "{name} constructs a raw Command; use proc::hidden_command instead"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn hidden_command_keeps_the_requested_program() {
        let command = super::hidden_command("nvidia-smi.exe");
        assert_eq!(command.get_program(), "nvidia-smi.exe");
    }
}
