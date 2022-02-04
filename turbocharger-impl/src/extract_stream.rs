pub fn inner_ty(orig_fn_ret_ty: syn::Type) -> Option<syn::Type> {
 let impltrait = match orig_fn_ret_ty {
  syn::Type::ImplTrait(impltrait) => Some(impltrait),
  _ => return None,
 };
 let bounds = match impltrait {
  Some(syn::TypeImplTrait { bounds, .. }) => Some(bounds),
  _ => return None,
 };
 let path = match bounds.unwrap()[0].clone() {
  syn::TypeParamBound::Trait(syn::TraitBound { path, .. }) => Some(path),
  _ => return None,
 };
 let segments = match path {
  Some(syn::Path { segments, .. }) => Some(segments),
  _ => return None,
 };
 let pathsegment = segments.unwrap()[0].clone();
 if pathsegment.ident != "Stream" {
  return None;
 }
 let anglebracketedgenericarguments = match pathsegment.arguments {
  syn::PathArguments::AngleBracketed(anglebracketedgenericarguments) => {
   Some(anglebracketedgenericarguments)
  }
  _ => return None,
 };
 let args = match anglebracketedgenericarguments {
  Some(syn::AngleBracketedGenericArguments { args, .. }) => Some(args),
  _ => return None,
 };
 let arg = args.unwrap()[0].clone();
 let binding = match arg {
  syn::GenericArgument::Binding(binding) => Some(binding),
  _ => return None,
 };
 let syn::Binding { ty, .. } = binding.unwrap();

 Some(ty)
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_extract_stream() {
  assert_eq!(
   inner_ty(syn::parse_str::<syn::Type>("impl Stream<Item = u32>").unwrap()),
   Some(syn::parse_str::<syn::Type>("u32").unwrap())
  );
  assert_eq!(
   inner_ty(syn::parse_str::<syn::Type>("impl Stream<Item = Result<i32, anyhow::Error>>").unwrap()),
   Some(syn::parse_str::<syn::Type>("Result<i32, anyhow::Error>").unwrap())
  );
  assert_eq!(
   inner_ty(syn::parse_str::<syn::Type>("impl Stream<Item = anyhow::Result<i32>>").unwrap()),
   Some(syn::parse_str::<syn::Type>("anyhow::Result<i32>").unwrap())
  );
  assert_eq!(inner_ty(syn::parse_str::<syn::Type>("u32").unwrap()), None);
 }
}
