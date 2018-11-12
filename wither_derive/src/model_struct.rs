use inflector::Inflector;
use syn;

use ::msg;

/// All `Model` struct attributes which have been accumulated from the target struct.
#[derive(Default)]
pub(crate) struct MetaModelStructData {
    /// The name to be used for the model's collection.
    pub collection_name: String,

    /// An attribute to control whether or not this derive macro will check for serde attributes.
    pub skip_serde_checks: bool,
}

impl MetaModelStructData {
    /// Extract needed data from the target model's struct attributes.
    pub fn new(attrs: &[syn::Attribute], target_ident: &syn::Ident) -> Self {
        // Collect the target's struct level `model` attrs.
        let mut data = attrs.iter().fold(MetaModelStructData::default(), |mut acc, attr| {
            // Ensure attr is structured properly.
            let meta = match attr.interpret_meta() {
                Some(meta) => meta,
                None => return acc,
            };
            let meta_name = meta.name().to_string();

            // If we are not looking at a `model` attr, then skip.
            match meta_name.as_str() {
                "model" => {
                    unpack_model_attr(&meta, &mut acc);
                    acc
                },
                _ => acc,
            }
        });

        // If collection name is default "", then use the struct's ident.
        if data.collection_name.len() == 0 {
            data.collection_name = target_ident.to_string().to_table_case().to_plural();
        }
        data
    }

}

/// Unpack the data from any struct level `model` attrs.
fn unpack_model_attr(meta: &syn::Meta, struct_data: &mut MetaModelStructData) {
    // Unpack the inner attr's components.
    match meta {
        // Model attr must be a list.
        syn::Meta::List(list) => list.nested.iter().by_ref()
            .filter_map(|nested_meta| match nested_meta {
                syn::NestedMeta::Meta(meta) => Some(meta),
                _ => panic!(msg::MODEL_ATTR_FORM),
            }).for_each(|innermeta| {
                match innermeta {
                    syn::Meta::Word(ident) => handle_ident_attr(ident, struct_data),
                    syn::Meta::NameValue(kv) => handle_kv_attr(kv, struct_data),
                    _ => panic!(msg::MODEL_STRUCT_ATTRS),
                }
            }),
        _ => panic!(msg::MODEL_ATTR_FORM),
    };
}

fn handle_kv_attr(kv: &syn::MetaNameValue, struct_data: &mut MetaModelStructData) {
    let ident = kv.ident.to_string();
    match &kv.lit {
        syn::Lit::Str(ref val) => {
            let value = val.value();
            match ident.as_str() {
                "collection_name" => {
                    struct_data.collection_name = value;
                    if struct_data.collection_name.len() < 1 {
                        panic!("The `Model` struct attribute 'collection_name' may not have a zero-length value.");
                    }
                },
                _ => panic!(format!("Unrecognized struct-level `Model` attribute '{}'.", ident)),
            }
        },
        _ => panic!("Only string literals are supported as named values in `Model` attributes."),
    }
}

fn handle_ident_attr(ident: &syn::Ident, struct_data: &mut MetaModelStructData) {
    let ident = ident.to_string();
    match ident.as_str() {
        "skip_serde_checks" => {
            struct_data.skip_serde_checks = true;
        },
        _ => panic!(format!("Unrecognized struct-level `Model` attribute '{}'.", ident)),
    }
}
