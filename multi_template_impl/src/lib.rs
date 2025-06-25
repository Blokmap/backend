use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, Ident, ItemEnum, LitStr};

#[proc_macro_derive(MultiTemplate, attributes(multi_template))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = syn::parse_macro_input!(input as ItemEnum);

	let template_structs: Vec<TokenStream> = input
		.variants
		.iter()
		.map(|v| {
			let multi_template_attribute: MultiTemplateAttribute;
			let variant_span: Span;

			if let Some(attr) = &v.attrs.first() {
				match parse_attr(attr) {
					Ok(m) => {
						multi_template_attribute = m;
						variant_span = attr.span();
					},
					Err(e) => return e.into_compile_error().into(),
				}
			} else {
				return span_error(v, "missing multi_template attribute")
					.into_compile_error();
			}

			let multi_struct_name = format_ident! {"{}MultiTemplate", v.ident};
			let multi_struct_fields: Vec<TokenStream> =
				multi_template_attribute
					.variants
					.iter()
					.map(|v| {
						let variant_name = Ident::new(&v, variant_span);

						quote! {
							#variant_name: String
						}
					})
					.collect::<Vec<_>>();

			let variant_enum_name = format_ident! {"{}Template", v.ident};
			let variant_enum_variants: Vec<TokenStream> =
				multi_template_attribute
					.variants
					.iter()
					.map(|var| {
						let variant_name = var.to_case(Case::Pascal);
						let variant_name =
							Ident::new(&variant_name, variant_span);

						let variant_fields =
							v.fields.clone().into_token_stream();

						let template_path = format!(
							"{}/{}.{}",
							multi_template_attribute.name,
							var,
							variant_to_ext(var),
						);

						quote! {
							#[template(path = #template_path)]
							#variant_name #variant_fields
						}
					})
					.collect();

			quote! {
				struct #multi_struct_name {
					#(#multi_struct_fields),*
				}

				#[derive(askama::Template)]
				enum #variant_enum_name {
					#(#variant_enum_variants),*
				}
			}
		})
		.collect();

	let rendered = quote! {
		#(#template_structs)*
	};

	proc_macro::TokenStream::from(rendered)
}

struct MultiTemplateAttribute {
	name:     String,
	variants: Vec<&'static str>,
}

fn variant_to_ext(var: &'static str) -> &'static str {
	match var {
		"email" => "html",
		"markdown" => "md",
		"text" => "txt",
		_ => unreachable!(),
	}
}

fn span_error<T: quote::ToTokens>(t: T, message: &str) -> Error {
	Error::new_spanned(t, message)
}

fn parse_attr(attr: &syn::Attribute) -> Result<MultiTemplateAttribute, Error> {
	// Asummes only one attribute
	let message = "expected `multi_template`";

	let mut name = String::new();
	let mut variants = vec![];

	if attr.path().is_ident("multi_template") {
		attr.parse_nested_meta(|meta| {
			if meta.path.is_ident("name") {
				let value = meta.value()?;
				let lit: LitStr = value.parse()?;
				name = lit.value();
				return Ok(());
			}

			if meta.path.is_ident("variants") {
				meta.parse_nested_meta(|meta| {
					if meta.path.is_ident("email") {
						variants.push("email");
						Ok(())
					} else if meta.path.is_ident("markdown") {
						variants.push("markdown");
						Ok(())
					} else if meta.path.is_ident("text") {
						variants.push("text");
						Ok(())
					} else {
						Err(meta.error("unknown variant"))
					}
				})?;

				return Ok(());
			}

			Err(meta.error("unrecognized attribute"))
		})
	} else {
		Err(span_error(attr, message))
	}?;

	Ok(MultiTemplateAttribute { name, variants })
}
