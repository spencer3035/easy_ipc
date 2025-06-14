use std::fmt::Display;

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type, parse::Parse};

#[proc_macro_derive(Model, attributes(easy_ipc))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let MessageAttributes {
        server_message,
        client_message,
    } = match parse_message_type(&input) {
        Ok(messages) => messages,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let name = &input.ident;

    let model_impl = quote! {
        impl ::easy_ipc::prelude::Model for #name {
            type ServerMsg = #server_message;
            type ClientMsg = #client_message;
            fn model() -> ::easy_ipc::prelude::ClientServerModel<Self::ClientMsg, Self::ServerMsg> {
                ::easy_ipc::model!()
            }
        }
    };

    TokenStream::from(model_impl)
}

const CRATE_NAME: &str = "easy_ipc";
const SERVER_MESSAGE: &str = "server_message";
const CLIENT_MESSAGE: &str = "client_message";

/// Gets (server_message, client_message)
fn parse_message_type(input: &DeriveInput) -> Result<MessageAttributes, syn::Error> {
    for attr in &input.attrs {
        let segments = &attr.path().segments;
        if segments.len() == 1 && segments[0].ident == CRATE_NAME {
            let tmp: MessageAttributes = attr.parse_args().map_err(|mut e| {
                e.combine(syn::Error::new(e.span(), DeriveError::GenericError));
                e
            })?;
            return Ok(tmp);
        }
    }

    Err(syn::Error::new_spanned(&input, DeriveError::GenericError))
}

struct MessageAttributes {
    server_message: syn::Type,
    client_message: syn::Type,
}

enum DeriveError {
    MissingServerMessage,
    MissingClientMessage,
    GenericError,
}

impl Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeriveError::MissingServerMessage => {
                write!(f, "missing {SERVER_MESSAGE} = YourServerMessage")
            }
            DeriveError::MissingClientMessage => {
                write!(f, "missing {CLIENT_MESSAGE} = YourClientMessage")
            }
            DeriveError::GenericError => {
                write!(
                    f,
                    "{CRATE_NAME} needs attributes defining server and client messages \n`#[{CRATE_NAME}({CLIENT_MESSAGE} = YourClientMessage, {SERVER_MESSAGE} = YourServerMessage)]`"
                )
            }
        }
    }
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
