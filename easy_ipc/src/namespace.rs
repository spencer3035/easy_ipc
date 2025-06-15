use std::path::{Path, PathBuf};

use interprocess::local_socket::{GenericNamespaced, NameType};

/// Gets a sensible default name of socket according to your OS and a final path name.
///
/// On linux it would be something like `"/run/user/1000/<filename>"`.
/// ```
/// use easy_ipc::namespace::default_namespace;
/// let my_socket = default_namespace("myapp.socket");
/// ```
pub fn default_namespace<P>(filename: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let mut path = default_socket_path();
    path.push(filename);
    // TODO: Delete debug prints
    if GenericNamespaced::is_supported() {
        println!("Using generic namespaced");
    } else {
        println!("Using file: {}", path.display());
    }
    path
}

// TODO: Implement for "macos" "ios" "android" "freebsd" "dragonfly" "openbsd" "netbsd" "none"
// should always fail, we need an os to do what this crate does.
#[cfg(not(target_family = "unix"))]
#[cfg(not(target_family = "windows"))]
fn default_socket_path() -> PathBuf {
    panic!("platform not supported")
}

#[cfg(target_family = "windows")]
fn default_socket_path() -> PathBuf {
    // Windows only supports pipes in namespaces
    PathBuf::new()
}

#[cfg(target_family = "unix")]
fn default_socket_path() -> PathBuf {
    use interprocess::local_socket::{GenericNamespaced, NameType};

    if GenericNamespaced::is_supported() {
        PathBuf::new()
    } else {
        default_socket_path_unix()
    }
}

#[cfg(target_family = "unix")]
fn default_socket_path_unix() -> PathBuf {
    let mut p = PathBuf::new();
    p.push("/run");
    p.push("user");
    p.push(users::get_current_uid().to_string());
    p
}
