use quote::quote;
use proc_macro::TokenStream;

// HelloWorld is the name for the derive
// hello_world_name is the name of our optional attribute
#[proc_macro_derive(HelloWorld, attributes(hello_world_name))]
pub fn hello_world(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the implementation
    impl_hello_world(&ast)
}

fn impl_hello_world(ast: &syn::DeriveInput) -> TokenStream {
    let identifier = &ast.ident;
    let identifier_str = identifier.to_string();
    // Use the name provided by the attribute
    // If there is no attribute, use the identifier
    let hello_world_name = get_name_attribute(ast).unwrap_or_else(|| identifier_str);
    let gen = quote! {
        // Insert an implementation for our trait
        impl HelloWorld for #identifier {
            fn hello_world() {
                println!(
                    "The struct or enum {} says: \"Hello world from {}!\"",
                    stringify!(#identifier),
                    #hello_world_name
                );
            }
        }
    };
    gen.into()
}

fn get_name_attribute(ast: &syn::DeriveInput) -> Option<String> {
    const ATTR_NAME: &str = "hello_world_name";

    // Go through all attributes and find one with our name
    if let Some(attr) = ast.attrs.iter().find(|a| a.path.is_ident(ATTR_NAME)) {
        // Check if it's in the form of a name-value pair
        if let syn::Meta::NameValue(pair) = attr.parse_meta().ok()? {
            // Check if the value is a string
            if let syn::Lit::Str(name) = pair.lit {
                Some(name.value())
            } else {
                panic!(
                    "Expected a string as the value of {}",
                    ATTR_NAME
                );
            }
        } else {
            panic!(
                "Expected an attribute in the form #[{} = \"Some value\"]",
                ATTR_NAME
            );
        }
    } else {
        None
    }
}
