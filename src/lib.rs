use interprocess::local_socket::{GenericFilePath, GenericNamespaced, Name, prelude::*};

pub mod client;
pub mod connection;
pub mod model;
mod packet;
pub mod server;

/// Gets the name/file of the socket
// TODO: Needs to be generated
pub fn default_socket_name() -> Name<'static> {
    if GenericNamespaced::is_supported() {
        "example.sock".to_ns_name::<GenericNamespaced>().unwrap()
    } else {
        "/home/spencer/example.sock"
            .to_fs_name::<GenericFilePath>()
            .unwrap()
    }
}
