use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::DeriveInput;
//use async_trait::async_trait;

struct Keys {
    pub pk: String,
    pub sk: Option<String>,
}

fn get_queries(
    table_name: &String,
    index: &Option<String>,
    partition_key: &String,
    sort_key: &Option<String>,
) -> syn::__private::TokenStream2 {
    let name = if index.is_some() {
        format_ident!(
            "query_{}",
            index.clone().unwrap().to_lowercase().replace("-", "_")
        )
    } else {
        format_ident!("query")
    };
    let index = if index.is_some() {
        Some(quote! {.index_name(#index)})
    } else {
        None
    };

    let expanded = if sort_key.is_some() {
        quote! {
            pub async fn #name<T, P, S>(&self, pk: P, sk: Option<S>) -> Result<Vec<T>, aws_lambda_api::error::BackendError>
                where
                    T: serde::de::DeserializeOwned,
                    P: serde::Serialize,
                    S: serde::Serialize
            {
                let mut query = self.client.query()
                    .table_name(#table_name)
                    #index
                    .key_condition_expression("#pk = :pk")
                    .expression_attribute_names("#pk", #partition_key)
                    .expression_attribute_values(":pk", serde_dynamo::to_attribute_value(pk).unwrap());

                if let Some(sk) = sk {
                    query = query.key_condition_expression("#pk = :pk and begins_with(#sk, :sk)")
                    .expression_attribute_names("#sk", #sort_key)
                    .expression_attribute_values(":sk", serde_dynamo::to_attribute_value(sk).unwrap());
                }
                // println!("Query {:?}", query);
                let response = query.send().await?;
                // println!("Raw {:?} response: {:?}", index, response);
                let data: Vec<T> = serde_dynamo::from_items(response.items.unwrap()).unwrap();
                Ok(data)
            }
        }
    } else {
        quote! {
            pub async fn #name<T, P, S>(&self, pk: P) -> Result<Vec<T>, aws_lambda_api::error::BackendError>
                where
                    T: serde::de::DeserializeOwned,
                    P: serde::Serialize,
            {
                let query = self.client.query()
                    .table_name(#table_name)
                    #index
                    .key_condition_expression("#pk = :pk")
                    .expression_attribute_names("#pk", #partition_key)
                    .expression_attribute_values(":pk", serde_dynamo::to_attribute_value(pk).unwrap());
                // println!("Query {:?}", query);
                let response = query.send().await?;
                // println!("Raw {:?} response: {:?}", index, response);
                let data: Vec<T> = serde_dynamo::from_items(response.items.unwrap()).unwrap();
                Ok(data)
            }
        }
    };
    expanded
    // TokenStream::from(expanded)
}

#[proc_macro_derive(DynamodbTable, attributes(dynamodb))]
pub fn dynamodb_read(input: TokenStream) -> TokenStream {
    // Parse the tokens into a syntax tree
    let ast: DeriveInput = syn::parse(input).unwrap();
    // Build the output, possibly using quasi-quotation
    let name = &ast.ident;
    let attrs = &ast.attrs;

    let mut table_name = None::<String>;
    let mut partition_key = None::<String>;
    let mut sort_key = None::<String>;
    let mut indexes: HashMap<String, Keys> = HashMap::new();

    for attr in attrs {
        if attr.path().is_ident("dynamodb") {
            let nested = attr.parse_args_with(syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated).unwrap();
            let mut is_table = false;
            let mut index = None::<String>;
            let mut pk = None::<String>;
            let mut sk = None::<String>;

            for meta in nested {
                match meta {
                    // #[dynamodb(table="test")]
                    syn::MetaNameValue {
                        ref path,
                        value:
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(token),
                                ..
                            }),
                        ..
                    } if path.is_ident("table") => {
                        table_name = Some(token.value());
                        is_table = true;
                    }
                    syn::MetaNameValue {
                        ref path,
                        value:
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(token),
                                ..
                            }),
                        ..
                    } if path.is_ident("index") => {
                        index = Some(token.value());
                    }
                    syn::MetaNameValue {
                        ref path,
                        value:
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(token),
                                ..
                            }),
                        ..
                    } if path.is_ident("pk") => {
                        pk = Some(token.value());
                    }
                    syn::MetaNameValue {
                        ref path,
                        value:
                            syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(token),
                                ..
                            }),
                        ..
                    } if path.is_ident("sk") => {
                        sk = Some(token.value());
                    }
                    _ => {
                        panic!("unrecognized repr {:?}", meta);
                    }
                }
            }
            if is_table {
                partition_key = pk;
                sort_key = sk;
            } else {
                indexes.insert(
                    index.unwrap(),
                    Keys {
                        pk: pk.unwrap(),
                        sk: sk,
                    },
                );
            }
        }
    }
    let query = get_queries(
        &table_name.clone().unwrap(),
        &None,
        &partition_key.unwrap(),
        &sort_key,
    );
    let indexes = indexes.into_iter().map(|(index, keys)| {
        get_queries(
            &table_name.clone().unwrap(),
            &Some(index),
            &keys.pk,
            &keys.sk,
        )
    });
    let expanded = quote! {
        impl #name {
            #query
            #(#indexes)*
        }
    };
    expanded.into()
}
