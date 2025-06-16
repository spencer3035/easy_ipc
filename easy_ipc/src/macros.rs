/// Generates a socket based on your crate name.
///
/// On linux it would be something like `"/run/user/1000/mycrate.socket"` if your crate name was
/// `mycrate`.
#[macro_export]
macro_rules! ipc_namespace {
    () => {{
        let name = ::std::string::ToString::to_string(::std::env!("CARGO_CRATE_NAME")) + ".sock";
        $crate::namespace::namespace(&name)
    }};
}

/// Expands the "CARGO_PKG_VERSION" environment variable, usually something like "1.2.3".
#[macro_export]
macro_rules! ipc_version_string {
    () => {
        ::std::env!("CARGO_PKG_VERSION")
    };
}

/// Create a new [`crate::prelude::ClientServerModel`] instance with sane defaults
///
/// This makes use of [`ipc_version_string`] to create a header magic bytes and [`ipc_namespace`] to name
/// the connection.
#[macro_export]
macro_rules! ipc_model {
    () => {
        $crate::prelude::ClientServerOptions::new($crate::ipc_namespace!())
            .magic_bytes($crate::ipc_version_string!())
            .create()
    };
}
