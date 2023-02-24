#![recursion_limit="128"]

use proc_macro2::TokenStream;
use syn::{Meta, MetaList, NestedMeta, Path};
use synstructure::decl_derive;
use quote::quote;

decl_derive!([Abomonation, attributes(unsafe_abomonate_ignore, unsafe_abomonate_proxy)] => derive_abomonation);

fn derive_abomonation(mut s: synstructure::Structure) -> proc_macro2::TokenStream {
    s.filter(|bi| {
        !bi.ast().attrs.iter()
            .map(|attr| attr.parse_meta())
            .filter_map(Result::ok)
            .any(|attr| attr.path().is_ident("unsafe_abomonate_ignore"))
    });

    let entomb = s.each(|bi| {
        let bi = wrap_with_proxy(bi);
        quote! {
            ::abomonation::Abomonation::entomb(#bi, _write)?;
        }
    });

    let extent = s.each(|bi| {
        let bi = wrap_with_proxy(bi);
        quote! {
            sum += ::abomonation::Abomonation::extent(#bi);
        }
    });

    s.bind_with(|_| synstructure::BindStyle::RefMut);

    let exhume = s.each(|bi| {
        let bi = unwrap_with_proxy(bi);
        quote! {
            let temp = bytes;
            bytes = ::abomonation::Abomonation::exhume(#bi, temp)?;
        }
    });

    s.bound_impl(quote!(abomonation::Abomonation), quote! {
        #[inline] unsafe fn entomb<W: ::std::io::Write>(&self, _write: &mut W) -> ::std::io::Result<()> {
            match *self { #entomb }
            Ok(())
        }
        #[allow(unused_mut)]
        #[inline] fn extent(&self) -> usize {
            let mut sum = 0;
            match *self { #extent }
            sum
        }
        #[allow(unused_mut)]
        #[inline] unsafe fn exhume<'a,'b>(
            &'a mut self,
            mut bytes: &'b mut [u8]
        ) -> Option<&'b mut [u8]> {
            match *self { #exhume }
            Some(bytes)
        }
    })
}

fn wrap_with_proxy(bi: &synstructure::BindingInfo) -> TokenStream {
    let proxy = get_proxy(bi);
    if let Some(path) = proxy {
        quote! { &#path::wrap(#bi) }
    } else {
        quote! { #bi }
    }
}

#[allow(dead_code)]
fn unwrap_with_proxy(bi: &synstructure::BindingInfo) -> TokenStream {
    let proxy = get_proxy(bi);
    if let Some(path) = proxy {
        quote! { &mut #path::unwrap(#bi) }
    } else {
        quote! { #bi }
    }
}

fn get_proxy(bi: &synstructure::BindingInfo) -> Option<Path> {
    bi.ast().attrs.iter()
        .map(|attr| attr.parse_meta())
        .filter_map(Result::ok)
        .filter(|attr| attr.path().is_ident("unsafe_abomonate_proxy"))
        .map(|attr| {
            if let Meta::List(MetaList {nested, ..}) = attr {
                let parts: Vec<NestedMeta> = nested.into_iter().collect();
                if parts.len() == 1 {
                    let first = parts.into_iter().nth(0).unwrap();
                    if let NestedMeta::Meta(Meta::Path(path)) = first {
                        return path;
                    }
                }
            }
            panic!("unsafe_abomonate_proxy needs a path");
        })
        .nth(0)
}
