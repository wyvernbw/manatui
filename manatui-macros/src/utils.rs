use quote::quote;

pub fn manatui_layout() -> proc_macro2::TokenStream {
    quote! {
        ::manatui::layout
    }
}
