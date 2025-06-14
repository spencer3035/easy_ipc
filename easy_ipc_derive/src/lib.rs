use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Ident, Type, parse::Parse};

#[proc_macro_derive(Model, attributes(easy_ipc))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let MessageAttributes {
        server_message,
        client_message,
    } = match parse_message_type(&input.attrs) {
        Ok(messages) => messages,
        Err(err) => {
            return syn::Error::new_spanned(&input, err)
                .to_compile_error()
                .into();
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
fn parse_message_type(attrs: &[Attribute]) -> Result<MessageAttributes, String> {
    println!("{:#?}", attrs);
    let mut server_msg_ty = None;
    let mut client_msg_ty = None;
    let server_attr_invalid =
        format!("Needs to have valid type in #[{CRATE_NAME}::{SERVER_MESSAGE} = \"...\"]");
    let client_attr_invalid =
        format!("Needs to have valid type in #[{CRATE_NAME}::{CLIENT_MESSAGE} = \"...\"]");
    for attr in attrs {
        let segments = &attr.path().segments;
        if segments.len() == 2 && segments[0].ident.to_string() == "easy_ipc" {
            let kind = &segments[1];
            println!("Got kind {kind:#?} ({})", kind.ident.to_string());
            match kind.ident.to_string().as_str() {
                "server_message" => {
                    let ty: Type = attr
                        .parse_args()
                        .map_err(|_| server_attr_invalid.to_string())?;
                    server_msg_ty = Some(ty);
                }
                "client_message" => {
                    let ty: Type = attr
                        .parse_args()
                        .map_err(|_| client_attr_invalid.to_string())?;
                    client_msg_ty = Some(ty);
                }

                _ => {}
            }
        }
    }

    let server_attr_missing =
        "Missing or invalid attribute, need #[easy_ipc::server_message = \"...\"]";
    let client_attr_missing =
        "Missing of invalid attribute, need #[easy_ipc::client_message = \"...\"]";

    let server_msg_ty = server_msg_ty.ok_or(server_attr_missing.to_string())?;
    let client_msg_ty = client_msg_ty.ok_or(client_attr_missing.to_string())?;

    println!("Got server: {:#?}", server_msg_ty);
    println!("Got client: {:#?}", client_msg_ty);
    Ok(MessageAttributes {
        server_message: server_msg_ty,
        client_message: client_msg_ty,
    })
}

struct MessageAttributes {
    server_message: syn::Type,
    client_message: syn::Type,
}

const NEED_BOTH_CLI_SER_ERROR: &str = "Expected ";

impl Parse for MessageAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first_ident: Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        let first_type: Type = input.parse()?;

        input.parse::<syn::Token![,]>()?;

        let second_ident: Ident = input.parse()?;
        input.parse::<syn::Token![=]>()?;
        let second_type: Type = input.parse()?;

        let (server_message, client_message) = match (
            first_ident.to_string().as_str(),
            second_ident.to_string().as_str(),
        ) {
            (CLIENT_MESSAGE, SERVER_MESSAGE) => (second_type, first_type),
            (SERVER_MESSAGE, CLIENT_MESSAGE) => (first_type, second_type),
            _ => return Err(input.error(NEED_BOTH_CLI_SER_ERROR)),
        };
        println!("first = {}", first_ident.to_string());

        Ok(MessageAttributes {
            server_message,
            client_message,
        })
    }
}
