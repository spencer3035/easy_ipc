use std::fmt::Display;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type, parse::Parse};

/// Crate name, used to namespace attributes and for diagnostic messages
const CRATE_NAME: &str = "easy_ipc";
/// The name of the server message attribute
const SERVER_MESSAGE: &str = "server_message";
/// The name of the client message attribute
const CLIENT_MESSAGE: &str = "client_message";

/// Derive the easy_ipc::prelude::Model trait
#[proc_macro_derive(IpcModel, attributes(easy_ipc))]
pub fn ipc_model_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Parse the message types from the attributes
    let MessageAttributes {
        server_message,
        client_message,
    } = match parse_message_type(&input) {
        Ok(messages) => messages,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    // Generate the appropriate implementation block
    let model_impl = quote! {
        impl ::easy_ipc::prelude::IpcModel for #name {
            type ServerMsg = #server_message;
            type ClientMsg = #client_message;
            fn model() -> ::easy_ipc::prelude::ClientServerModel<Self::ClientMsg, Self::ServerMsg> {
                ::easy_ipc::ipc_model!()
            }
        }
    };

    TokenStream::from(model_impl)
}

/// Parses the attributes from the appropriate information, returns an error if parsing fails
fn parse_message_type(input: &DeriveInput) -> Result<MessageAttributes, syn::Error> {
    for attr in &input.attrs {
        let segments = &attr.path().segments;
        // Should only be one segment, and name should match namespace
        if segments.len() == 1 && segments[0].ident == CRATE_NAME {
            let attr: MessageAttributes = attr.parse_args().map_err(|mut e| {
                e.combine(syn::Error::new(e.span(), DeriveError::GenericError));
                e
            })?;
            return Ok(attr);
        }
    }

    // Didn't find required attributes
    Err(syn::Error::new_spanned(&input, DeriveError::GenericError))
}

/// Errors that can happen
enum DeriveError {
    MissingServerMessage,
    MissingClientMessage,
    GenericError,
}

impl DeriveError {
    /// Gets the default attribute implementation
    fn default_useage() -> String {
        format!(
            "#[{CRATE_NAME}({CLIENT_MESSAGE} = YourClientMessage, {SERVER_MESSAGE} = YourServerMessage)]"
        )
    }

    /// Get the client portion of the attribute
    fn client_usage() -> String {
        format!("{CLIENT_MESSAGE} = YourClientMessage")
    }

    /// Get the client portion of the attribute
    fn server_usage() -> String {
        format!("{SERVER_MESSAGE} = YourServerMessage")
    }
}

impl Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeriveError::MissingServerMessage => {
                write!(f, "missing {}", DeriveError::server_usage())
            }
            DeriveError::MissingClientMessage => {
                write!(f, "missing {}", DeriveError::client_usage())
            }
            DeriveError::GenericError => {
                write!(
                    f,
                    "invalid or missing attributes for `#[derive(Model)]` from {CRATE_NAME}\nusage: {}",
                    DeriveError::default_useage()
                )
            }
        }
    }
}

/// Helper struct to parse the attributes
struct MessageAttributes {
    server_message: syn::Type,
    client_message: syn::Type,
}

impl Parse for MessageAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse "first_ident = first_type"
        let first_ident: Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        let first_type: Type = input.parse()?;

        // Comma separated
        input.parse::<syn::Token![,]>()?;

        // Parse "second_ident = second_type"
        let second_ident: Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        let second_type: Type = input.parse()?;

        // Split them into server/client messages depending on the identifiers
        let (server_message, client_message) = match (
            first_ident.to_string().as_str(),
            second_ident.to_string().as_str(),
        ) {
            (CLIENT_MESSAGE, SERVER_MESSAGE) => (second_type, first_type),
            (SERVER_MESSAGE, CLIENT_MESSAGE) => (first_type, second_type),
            _ => {
                let has_server_message = first_ident.to_string() == SERVER_MESSAGE
                    || second_ident.to_string() == SERVER_MESSAGE;
                let has_client_message = first_ident.to_string() == CLIENT_MESSAGE
                    || second_ident.to_string() == CLIENT_MESSAGE;

                if !has_server_message {
                    return Err(input.error(DeriveError::MissingServerMessage));
                }
                if !has_client_message {
                    return Err(input.error(DeriveError::MissingClientMessage));
                }
                unreachable!();
            }
        };

        Ok(MessageAttributes {
            server_message,
            client_message,
        })
    }
}
