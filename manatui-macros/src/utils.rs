use quote::quote;

pub fn manatui_elemental() -> proc_macro2::TokenStream {
    quote! {
        ::manatui::manatui_elemental
    }
}
