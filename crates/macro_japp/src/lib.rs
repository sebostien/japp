extern crate proc_macro;

use glob::glob;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, quote};
use syn::{ItemFn, LitStr};

/// Generate test for each input file matching the glob.
///
/// From: <https://internals.rust-lang.org/t/dynamic-tests-revisited/18095/5>
///
/// # Example
/// ```rust
///# use macro_japp::test_glob;
/// #[test_glob("*.txt")]
/// fn parse_test_file(p: &std::path::Path) {
///     assert!(p.is_file());
/// }
/// ```
#[proc_macro_attribute]
pub fn test_glob(attr: TokenStream, item: TokenStream) -> TokenStream {
    let glob_lit: LitStr = syn::parse(attr).expect("A single glob pattern should be provided");

    let test: ItemFn = syn::parse(item).expect("Could not parse function");
    let util_fn_name = &test.sig.ident;

    let mut out = TokenStream::from(test.to_token_stream());

    let call_file = proc_macro2::Span::call_site().local_file().unwrap();
    let call_dir = call_file.parent().unwrap().to_path_buf();

    let glob_path = {
        let mut x = call_dir.clone();
        x.push(&glob_lit.value());
        x.to_str().unwrap().to_string()
    };

    for path in glob(&glob_path).expect("Could not parse glob") {
        let path = path.unwrap();
        let test_name = path.with_extension("").to_str().unwrap().replace("/", "_");
        let path_str = path
            .to_str()
            .unwrap()
            // This is some stupid shit!
            // TODO: Find a way to glob from crate root instead of workspace.
            //       Currently files yield relative to workspace but tests read from crate.
            //       Easiest to just impl own glob
            .trim_start_matches(&call_dir.parent().unwrap().to_string_lossy().to_string())
            .trim_start_matches("/");

        let gen_name = Ident::new(&test_name, Span::call_site());

        out.extend(TokenStream::from(quote! {
            #[test]
            fn #gen_name() {
                #util_fn_name(&std::path::Path::new(#path_str));
            }
        }));
    }

    out
}
