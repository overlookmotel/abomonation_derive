#![recursion_limit="128"]

use syn::{Meta, MetaList, NestedMeta, Path};
use synstructure::decl_derive;
use quote::quote;

decl_derive!([Abomonation, attributes(unsafe_abomonate_ignore, unsafe_abomonate_with)] => derive_abomonation);

fn derive_abomonation(mut s: synstructure::Structure) -> proc_macro2::TokenStream {
    s.filter(|bi| {
        !bi.ast().attrs.iter()
            .map(|attr| attr.parse_meta())
            .filter_map(Result::ok)
            .any(|attr| attr.path().is_ident("unsafe_abomonate_ignore"))
    });

    let entomb = s.each(|bi| {
        match get_with(bi) {
            Some(proxy) => quote! {
                #proxy::entomb_with(#bi, _write)?;
            },
            None => quote! {
                ::abomonation::Abomonation::entomb(#bi, _write)?;
            }
        }
    });

    let extent = s.each(|bi| {
        match get_with(bi) {
            Some(proxy) => quote! {
                sum += #proxy::extent_with(#bi);
            },
            None => quote! {
                sum += ::abomonation::Abomonation::extent(#bi);
            }
        }
    });

    s.bind_with(|_| synstructure::BindStyle::RefMut);

    let exhume = s.each(|bi| {
        match get_with(bi) {
            Some(proxy) => quote! {
                let temp = bytes;
                bytes = #proxy::exhume_with(#bi, temp)?;
            },
            None => quote! {
                let temp = bytes;
                bytes = ::abomonation::Abomonation::exhume(#bi, temp)?;
            }
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

fn get_with(bi: &synstructure::BindingInfo) -> Option<Path> {
    bi.ast().attrs.iter()
        .map(|attr| attr.parse_meta())
        .filter_map(Result::ok)
        .filter(|attr| attr.path().is_ident("unsafe_abomonate_with"))
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
            panic!("unsafe_abomonate_with needs a path");
        })
        .nth(0)
}
