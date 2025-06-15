use crate::model::ClientServerModel;

use {
    serde::{Deserialize, Serialize},
    std::path::Path,
};

/// Trys to cleanup the socket file if it exists.
///
/// `Ok(true)` if the socket was sucessfully removed
/// `Ok(false)` if there wasn't a socket file to remove
/// `Err(_)` if the cleanup failed for an io reason.
fn remove_socket_file<P>(socket: P) -> Result<bool, std::io::Error>
where
    P: AsRef<Path>,
{
    if socket.as_ref().try_exists()? {
        std::fs::remove_file(socket)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Tries to clean up a given file and prints errors if it fails.
///
/// Meant to be used in handling panics and signals sent to kill the program.
pub fn cleanup<P>(path: P)
where
    P: AsRef<Path>,
{
    match remove_socket_file(&path) {
        // Should be the usual case, program was exited while server was running, so we need to
        // clean up the socket
        Ok(true) => (),
        // Less common, the either the server was dropped or something bad might have happened
        // before the server was created.
        Ok(false) => (),
        // Bad, we failed deleting the socket file, this might lead to a zombie socket file or it
        // could be because of bad permissions
        Err(e) => eprintln!(
            "Couldn't clean up socket file {}: {e}",
            &path.as_ref().display()
        ),
    }
}

#[cfg(target_family = "windows")]
fn handle_os_signals<P>(path: P) -> Result<(), std::io::Error>
where
    P: AsRef<Path>,
{
    // TODO:
    Ok(())
}
/// Handles os signals by calling the cleanup function
#[cfg(target_family = "unix")]
fn handle_os_signals<P>(path: P) -> Result<(), std::io::Error>
where
    P: AsRef<Path>,
{
    use signal_hook::{consts::*, iterator::Signals};
    // Handle all term signals
    let mut signals = Signals::new(TERM_SIGNALS)?;
    for sig in signals.forever() {
        cleanup(path);
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
            libc::raise(sig);
        }
        // Failsafe exit in case the re-raise of the signals doesn't properly exit
        std::process::exit(1);
    }
    Ok(())
}

/// Sets up handlers to try and delete a given path upon panic and signals that ask to terminate
/// the process.
pub(crate) fn setup_handlers<C, S>(model: &ClientServerModel<C, S>)
where
    C: Serialize + for<'de> Deserialize<'de>,
    S: Serialize + for<'de> Deserialize<'de>,
{
    let path = &model.options.options_inner.socket_name;

    // Handle panics, we do this first because the handling of OS errors thread might panic
    let default_panic_hook = std::panic::take_hook();
    let path_clone = path.clone();
    std::panic::set_hook(Box::new(move |info| {
        cleanup(&path_clone);
        default_panic_hook(info)
    }));

    // Handle signals from the OS
    let path_clone = path.clone();
    std::thread::spawn(move || {
        if let Err(e) = handle_os_signals(&path_clone) {
            panic!("Failed setting up signal handlers: {e}");
        }
        // TODO: Windows hits here because handle_os_signals exits
        panic!("Stopped handling signals.");
    });
}
