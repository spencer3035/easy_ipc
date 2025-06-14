use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Type, parse_macro_input};

#[proc_macro_derive(Model, attributes(server_message, client_message))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let (server_message, client_message) = match parse_message_type(&input.attrs) {
        Ok(msg) => msg,
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

/// Gets (server_message, client_message)
fn parse_message_type(attrs: &[Attribute]) -> Result<(syn::Type, syn::Type), String> {
    let mut server_msg_ty = None;
    let mut client_msg_ty = None;
    let server_attr_invalid = "Needs to have valid type in #[server_message = \"...\"]";
    let client_attr_invalid = "Needs to have valid type in #[client_message = \"...\"]";
    for attr in attrs {
        if attr.path().is_ident("server_message") {
            let ty: Type = attr
                .parse_args()
                .map_err(|_| server_attr_invalid.to_string())?;
            server_msg_ty = Some(ty);
        };
        if attr.path().is_ident("client_message") {
            let ty: Type = attr
                .parse_args()
                .map_err(|_| client_attr_invalid.to_string())?;
            client_msg_ty = Some(ty);
        }
    }

    let server_attr_missing = "Missing #[server_message = \"...\"]";
    let client_attr_missing = "Missing #[client_message = \"...\"]";

    let server_msg_ty = server_msg_ty.ok_or(server_attr_missing.to_string())?;
    let client_msg_ty = client_msg_ty.ok_or(client_attr_missing.to_string())?;

    Ok((server_msg_ty, client_msg_ty))
}
