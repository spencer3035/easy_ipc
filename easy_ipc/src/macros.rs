/// Calls [`crate::namespace::namespace`] with `env!("CARGO_CRATE_NAME")`.
#[macro_export]
macro_rules! ipc_namespace {
    () => {{
        let name = ::std::string::ToString::to_string(::std::env!("CARGO_CRATE_NAME"));
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

/// Create a new [`crate::prelude::ClientServerModel`] instance with sane defaults.
///
/// This makes use of [`ipc_version_string`] to create header magic bytes and [`ipc_namespace`] to
/// name the connection.
///
/// # Warning!
///
/// This panics at runtime if the default namespace cannot be resolved.
#[macro_export]
macro_rules! ipc_model {
    () => {
        Ok(
            $crate::prelude::ClientServerOptions::new($crate::ipc_namespace!()?)
                .magic_bytes($crate::ipc_version_string!())
                .create(),
        )
    };
}
