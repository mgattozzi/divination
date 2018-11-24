extern crate proc_macro;
use self::proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Attribute, ItemStruct, ItemImpl, Field, punctuated::Punctuated, token::Comma};
use std::{fs, path::PathBuf};

#[proc_macro_attribute]
pub fn divination(args: TokenStream, input: TokenStream) -> TokenStream {

    let item_struct = parse_macro_input!(input as ItemStruct);
    let path = create_and_check_path(args);

    let (istruct, iimpl) = derive_struct(path, item_struct);
    TokenStream::from(quote!(
        #[macro_use]
        extern crate serde_derive;
        use std::collections::HashMap;
        use std::error;
        #istruct
        #iimpl
    ))
}

fn create_and_check_path(args: TokenStream) -> PathBuf {
    let args = args.to_string().split_whitespace().collect::<String>();

    if args.is_empty() {
        panic!("You need to provide a path to the attribute like so: \n#[divination(relative/crate/path)");
    }

    let mut path = PathBuf::new();
    path.push(args);

    if !path.exists() {
        panic!("File `{}` does not exist. Please provide a valid path", path.to_str().unwrap());
    }

    path
}

fn derive_struct(path: PathBuf, item_struct: ItemStruct) -> (ItemStruct, ItemImpl) {
    let attrs = item_struct.attrs;
    let ident = item_struct.ident;
    let vis = item_struct.vis;

    // We've guaranteed the file exists
    let config = toml::from_str::<toml::Value>(&fs::read_to_string(&path).unwrap()).unwrap();

    let mut fields: Punctuated<Field,Comma> = Punctuated::new();

    for (key, _) in config.as_table().expect("fuck") {
        fields.push(Field {
            attrs: Vec::new(),
            vis: parse_quote!(pub),
            ident: Some(Ident::new(&key, Span::call_site())),
            colon_token: Some(parse_quote!(:)),
            ty: parse_quote!{HashMap<String, toml::Value>}
        });
    }

    let derive: Attribute = parse_quote!(#[derive(Serialize, Deserialize)]);
    let mut item: ItemStruct = parse_quote! {
        #derive
        #vis struct #ident {
            #fields
        }
    };
    // Add the attributes like derive etc back in
    item.attrs.extend(attrs);

    let path = path.to_str();
    let item_impl: ItemImpl = parse_quote! {
        impl #ident {
            pub fn parse() -> Result<Self, Box<dyn error::Error>> {
                use std::fs;
                Ok(toml::from_str(&fs::read_to_string(#path)?)?)
            }
        }
    };

    (item, item_impl)
}
