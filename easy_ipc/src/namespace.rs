use std::path::{Path, PathBuf};

use interprocess::local_socket::{GenericNamespaced, NameType};

use crate::error::InitError;

/// Tries to get a sensible default path according to your OS and a name you pass in.
///
/// In general you should not pass a full path here. If you want to specify the exact path, just
/// pass it in directly when defining [`crate::model::IpcModel::model`].
///
/// In general, [`GenericNamespaced`] names are available and will work well. We fall back onto
/// file system paths if that fails. We use the [`dirs::data_dir`] internally and append your
/// namespace to it. It is rare that this will return `None`, it is generally fine to `.unwrap()`
/// or `.expect("...")` this function. If you need more sophisticated error handling file creation,
/// you should craft it up yourself when implementing your [`crate::model::IpcModel`].
///
/// ```
/// use easy_ipc::namespace::namespace;
/// let my_socket = namespace("myapp");
/// ```
///
/// Uses [`filesystem_path`] internally.
pub fn namespace<P>(namespace: P) -> Result<PathBuf, InitError>
where
    P: AsRef<Path>,
{
    let use_namespaced = GenericNamespaced::is_supported()
        && namespace.as_ref().iter().count() == 1
    // For some reason, Mac seems to struggle with cleaning up GenericNamespaced connections.
    // When I exit a program with ctrl-c, I get SocketAlreadyExists errors on subsequent launches.
    // This doesn't happen on Linux from my testing. We just banish Mac to the file system socket
    // corner because it wants to be special.
        && cfg!(not(target_os = "macos"));

    if use_namespaced {
        Ok(namespace.as_ref().to_path_buf())
    } else {
        filesystem_path(namespace)
    }
}

/// Makes a file system path based on the host OS and the name you give.
///
/// Note that this also has a side effect of creating the directory to put the file in.
///
/// ```
/// use easy_ipc::namespace::filesystem_path;
/// use dirs::data_dir;
/// # use std::path::PathBuf;
///
/// let app_name = "my_app";
/// let name = filesystem_path(app_name).unwrap();
///
/// let mut expected = data_dir().unwrap();
/// expected.push(app_name);
/// expected.push(app_name.to_string() + ".sock");
/// assert_eq!(name, expected);
/// ```
pub fn filesystem_path<P>(namespace: P) -> Result<PathBuf, InitError>
where
    P: AsRef<Path>,
{
    let mut path = dirs::data_dir().ok_or(InitError::FailedGettingNamespace)?;
    path.push(&namespace);

    // Try to make the directory here to make downstream stuff easier.
    if !path
        .try_exists()
        .map_err(|e| InitError::FailedConnectingToSocket(e))?
    {
        std::fs::create_dir(&path).map_err(|e| InitError::FailedConnectingToSocket(e))?;
    }
    path.push(namespace.as_ref().with_extension("sock"));
    Ok(path)
}
