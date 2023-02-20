/*
 *    Copyright  2023.  Fritz Ochsmann
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[cfg(feature = "skytable")]
#[proc_macro_derive(IntoSkyhashBytes)]
pub fn into_sky_hash_bytes_macro_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl skytable::types::IntoSkyhashBytes for &#name {
            fn as_bytes(&self) -> Vec<u8> {
                serde_json::to_string(self).unwrap().into_bytes()
            }
        }
    };

    expanded.into()
}

#[cfg(feature = "skytable")]
#[proc_macro_derive(FromSkyhashBytes)]
pub fn from_sky_hash_bytes_macro_derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl skytable::types::FromSkyhashBytes for #name {
            fn from_element(element: skytable::Element) -> skytable::SkyResult<Self> {
                let value: String = element.try_element_into()?;
                match serde_json::from_str(&value) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(skytable::error::Error::ParseError(value)),
                }
            }
        }
    };

    expanded.into()
}
