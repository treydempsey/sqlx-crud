use inflector::Inflector;
use proc_macro::{self, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, Attribute, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Ident,
    Lit, LitStr, Meta, MetaNameValue,
};

#[proc_macro_derive(SqlxCrud, attributes(database, id))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);
    match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => {
            let crate_name = crate_name();
            let static_model_schema = build_static_model_schema(&ident, &named);
            let sqlx_crud_impl = build_sqlx_crud_impl(&attrs, &ident, &named);

            quote! {
                #[automatically_derived]
                use #crate_name::traits::{Crud, Schema};

                #static_model_schema

                #sqlx_crud_impl
            }
            .into()
        }
        _ => panic!("this derive macro only works on structs with named fields"),
    }
}

fn build_static_model_schema(ident: &Ident, named: &Punctuated<Field, Comma>) -> TokenStream2 {
    let crate_name = crate_name();
    let model_schema_ident = model_schema_ident(ident);
    let table_name = table_name(ident);

    let id_column = id_column_ident(named).to_string();
    let columns_len = named.iter().count();
    let columns = named
        .iter()
        .flat_map(|f| &f.ident)
        .map(|f| LitStr::new(format!("{}", f).as_str(), f.span()));

    let sql_queries = build_sql_queries(ident, named);

    quote! {
        #[automatically_derived]
        static #model_schema_ident: #crate_name::schema::Metadata<'static, #columns_len> = #crate_name::schema::Metadata {
            table_name: #table_name,
            id_column: #id_column,
            columns: [#(#columns),*],
            #sql_queries
        };
    }
}

fn build_sql_queries(ident: &Ident, named: &Punctuated<Field, Comma>) -> TokenStream2 {
    let table_name = table_name(ident);
    let id_column_ident = id_column_ident(named);
    let insert_sql_binds = (0..named.iter().count())
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let update_sql_binds = named
        .iter()
        .flat_map(|f| &f.ident)
        .filter(|i| *i != id_column_ident)
        .map(|i| format!("{} = ?", i))
        .collect::<Vec<_>>()
        .join(", ");

    // XXX TODO XXX
    // Quote identifiers
    // Qualify columns with table identifiers
    let column_list_idents = named.iter().map(|f| &f.ident);
    let column_list = format!("{}", quote! {#(#column_list_idents), *});
    let select_sql = format!("SELECT {} FROM {}", column_list, table_name);
    let select_by_id_sql = format!(
        "SELECT {} FROM {} WHERE {} = ? LIMIT 1",
        column_list, table_name, id_column_ident
    );
    // XXX TODO XXX Explictly list columns `INTO {} (columns...,) VALUES`
    let insert_sql = format!("INSERT INTO {} VALUES ({})", table_name, insert_sql_binds);
    let update_by_id_sql = format!(
        "UPDATE {} SET {} WHERE {} = ?",
        table_name, update_sql_binds, id_column_ident
    );
    let delete_by_id_sql = format!("DELETE FROM {} WHERE {} = ?", table_name, id_column_ident);

    quote! {
        select_sql: #select_sql,
        select_by_id_sql: #select_by_id_sql,
        insert_sql: #insert_sql,
        update_by_id_sql: #update_by_id_sql,
        delete_by_id_sql: #delete_by_id_sql,
    }
}

fn build_sqlx_crud_impl(
    attrs: &[Attribute],
    ident: &Ident,
    named: &Punctuated<Field, Comma>,
) -> TokenStream2 {
    let crate_name = crate_name();
    let model_schema_ident = model_schema_ident(ident);
    let id_column_ident = id_column_ident(named);
    let id_ty = named
        .iter()
        .find(|f| f.ident.as_ref() == Some(id_column_ident))
        .map(|f| &f.ty)
        .expect("the id type");

    let insert_binds = named
        .iter()
        .flat_map(|f| &f.ident)
        .map(|i| quote! { .bind(&self.#i) });
    let update_binds = named
        .iter()
        .flat_map(|f| &f.ident)
        .filter(|i| *i != id_column_ident)
        .map(|i| quote! { .bind(&self.#i) });

    let db_ty = db_type(attrs);

    quote! {
            #[automatically_derived]
            impl #crate_name::traits::Schema for #ident {
                type Id = #id_ty;

                fn table_name() -> &'static str {
                    #model_schema_ident.table_name
                }

                fn id(&self) -> Self::Id {
                    self.#id_column_ident
                }

                fn id_column() -> &'static str {
                    #model_schema_ident.id_column
                }

                fn columns() -> &'static [&'static str] {
                    &#model_schema_ident.columns
                }

                fn select_sql() -> &'static str {
                    #model_schema_ident.select_sql
                }

                fn select_by_id_sql() -> &'static str {
                    #model_schema_ident.select_by_id_sql
                }

                fn insert_sql() -> &'static str {
                    #model_schema_ident.insert_sql
                }

                fn update_by_id_sql() -> &'static str {
                    #model_schema_ident.update_by_id_sql
                }

                fn delete_by_id_sql() -> &'static str {
                    #model_schema_ident.delete_by_id_sql
                }
            }

            #[automatically_derived]
            impl<'e> #crate_name::traits::Crud<'e, &'e ::sqlx::pool::Pool<#db_ty>> for #ident {
                fn insert_binds(
                    &'e self,
                    query: ::sqlx::query::Query<'e, ::sqlx::Sqlite, ::sqlx::sqlite::SqliteArguments<'e>>
                ) -> ::sqlx::query::Query<'e, ::sqlx::Sqlite, ::sqlx::sqlite::SqliteArguments<'e>> {
                    query
                        #(#insert_binds)*
                }

                fn update_binds(
                    &'e self,
                    query: ::sqlx::query::Query<'e, ::sqlx::Sqlite, ::sqlx::sqlite::SqliteArguments<'e>>
                ) -> ::sqlx::query::Query<'e, ::sqlx::Sqlite, ::sqlx::sqlite::SqliteArguments<'e>> {
                    query
                        #(#update_binds)*
                        .bind(&self.#id_column_ident)
                }
            }
        }
}

fn db_type(attrs: &[Attribute]) -> TokenStream2 {
    match attrs
        .iter()
        .find(|a| a.path.is_ident("database"))
        .map(|a| a.parse_meta())
    {
        Some(Ok(Meta::NameValue(MetaNameValue {
            lit: Lit::Str(s), ..
        }))) => match &*s.value() {
            "Any" => quote! { ::sqlx::Any },
            "Mssql" => quote! { ::sqlx::Mssql },
            "MySql" => quote! { ::sqlx::MySql },
            "Postgres" => quote! { ::sqlx::Postgres },
            "Sqlite" => quote! { ::sqlx::Sqlite },
            _ => panic!("unknown #[database] type {}", &s.value()),
        },
        _ => quote! { ::sqlx::Sqlite },
    }
}

fn model_schema_ident(ident: &Ident) -> Ident {
    format_ident!("{}_SCHEMA", ident.to_string().to_screaming_snake_case())
}

fn table_name(ident: &Ident) -> String {
    ident.to_string().to_table_case()
}

fn id_column_ident(named: &Punctuated<Field, Comma>) -> &Ident {
    // Search for a field with the #[id] attribute
    let id_attr = &named
        .iter()
        .find(|f| f.attrs.iter().any(|a| a.path.is_ident("id")))
        .map(|f| f.ident.as_ref())
        .flatten();

    // Otherwise default to the first field as the "id" column
    id_attr.unwrap_or_else(|| {
        named
            .iter()
            .flat_map(|f| &f.ident)
            .next()
            .expect("the first field")
    })
}

fn crate_name() -> TokenStream2 {
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let is_doctest = std::env::vars().any(|(k, _)| {
        k == "UNSTABLE_RUSTDOC_TEST_LINE" || k == "UNSTABLE_RUSTDOC_TEST_PATH"
    });
    if !is_doctest && crate_name == "sqlx-crud" {
        quote! { crate }
    } else {
        quote! { ::sqlx_crud }
    }
}
