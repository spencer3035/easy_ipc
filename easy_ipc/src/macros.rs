/// Generates a socket based on your crate name.
///
/// On linux it would be something like `"/run/user/1000/mycrate.socket"` if your crate name was
/// `mycrate`.
/// ```
/// use easy_ipc::socket_name;
/// let name = socket_name!();
/// ```
#[macro_export]
macro_rules! socket_name {
    () => {{
        let name = ::std::string::ToString::to_string(::std::env!("CARGO_CRATE_NAME")) + ".sock";
        $crate::namespace::default_namespace(&name)
    }};
}

/// Expands the "CARGO_PKG_VERSION" environment variable, usually something like "1.2.3".
#[macro_export]
macro_rules! version_string {
    () => {
        ::std::env!("CARGO_PKG_VERSION")
    };
}

/// Create a new [`ClientServerModel`] instance with sane defaults
///
/// This makes use of [`version_string`] to create a header magic bytes and [`socket_name`] to name
/// the connection.
#[macro_export]
macro_rules! model {
    () => {
        $crate::prelude::ClientServerOptions::new($crate::socket_name!())
            .magic_bytes($crate::version_string!())
            .create()
    };
}
