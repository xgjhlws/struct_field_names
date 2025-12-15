use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, Attribute, DeriveInput, Fields,
    Variant, Visibility,
};

#[proc_macro_derive(StructFieldNames, attributes(struct_field_names))]
pub fn derive_field_names(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let (vis, ident, generics) = (&ast.vis, &ast.ident, &ast.generics);
    let names_struct_ident = format_ident!("{}FieldStaticStr", ident.to_string());

    let fields = filter_fields(match ast.data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("FieldNames can only be derived for structs"),
    });

    let names_struct_fields = fields.iter().map(|(vis, ident)| {
        quote! {
            #vis #ident: &'static str
        }
    });

    let names_const_fields = fields.iter().map(|(_vis, ident)| {
        let ident_name = ident.to_string();
        let doc_comment = format!("field name for `{ident_name}`");
        quote! {
            #[doc = #doc_comment]
            #ident: #ident_name
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let tokens = quote! {
        #vis struct #names_struct_ident {
            #(#names_struct_fields),*
        }

        impl #impl_generics #ident #ty_generics
            #where_clause
        {

            #vis const FIELD_NAMES: #names_struct_ident = #names_struct_ident {
                #(#names_const_fields),*
            };
        }
    };
    tokens.into()
}

#[proc_macro_derive(EnumVariantNames, attributes(enum_variant_names))]
pub fn derive_variant_names(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let (vis, ident, generics) = (&ast.vis, &ast.ident, &ast.generics);
    let names_struct_ident = format_ident!("{}VariantsStaticStr", ident.to_string());

    let variants = filter_variants(match ast.data {
        syn::Data::Enum(ref e) => &e.variants,
        _ => panic!("VariantNames can only be derived for enums"),
    });

    let names_struct_fields = variants.iter().map(|ident| {
        quote! {
            #ident: &'static str
        }
    });

    let names_const_fields = variants.iter().map(|ident| {
        let ident_name = ident.to_string();
        quote! {
            #ident: #ident_name
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let tokens = quote! {
        #[allow(non_snake_case)]
        #vis struct #names_struct_ident {
            #(#names_struct_fields),*
        }

        impl #impl_generics #ident #ty_generics
            #where_clause
        {
            #vis const VARIANT_NAMES: #names_struct_ident = #names_struct_ident {
                #(#names_const_fields),*
            };
        }
    };
    tokens.into()
}

fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident)> {
    fields
        .iter()
        .filter_map(|field| {
            if !field
                .attrs
                .iter()
                .any(|attr| has_skip_attr(attr, "struct_field_names"))
                && field.ident.is_some()
            {
                let field_vis = field.vis.clone();
                let field_ident = field.ident.as_ref().unwrap().clone();
                Some((field_vis, field_ident))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn filter_variants(variants: &Punctuated<Variant, Comma>) -> Vec<Ident> {
    variants
        .iter()
        .filter_map(|variant| {
            if !variant
                .attrs
                .iter()
                .any(|attr| has_skip_attr(attr, "enum_variant_names"))
            {
                let ident = variant.ident.clone();
                Some(ident)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

const ATTR_META_SKIP: &str = "skip";

fn has_skip_attr(attr: &Attribute, path: &'static str) -> bool {
    if !attr.path().is_ident(path) {
        return false;
    }

    let mut has_skip = false;
    let result = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident(ATTR_META_SKIP) {
            has_skip = true;
        }
        Ok(())
    });

    result.is_ok() && has_skip
}
