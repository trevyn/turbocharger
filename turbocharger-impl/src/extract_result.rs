pub fn inner_ty(orig_fn_ret_ty: syn::Type) -> Option<syn::GenericArgument> {
 let typepath = match orig_fn_ret_ty {
  syn::Type::Path(typepath) => Some(typepath),
  _ => None,
 };
 let path = match typepath {
  Some(syn::TypePath { path, .. }) => Some(path),
  _ => None,
 };
 let pair = match path {
  Some(syn::Path { mut segments, .. }) => segments.pop(),
  _ => None,
 };
 let (is_result, arguments) = match pair.map(|pair| pair.into_value()) {
  Some(syn::PathSegment { ident, arguments }) => (ident == "Result", Some(arguments)),
  _ => (false, None),
 };
 if !is_result {
  return None;
 }
 let anglebracketedgenericarguments = match arguments {
  Some(syn::PathArguments::AngleBracketed(anglebracketedgenericarguments)) => {
   Some(anglebracketedgenericarguments)
  }
  _ => None,
 };
 let args = match anglebracketedgenericarguments {
  Some(syn::AngleBracketedGenericArguments { args, .. }) => Some(args),
  _ => None,
 };
 args.map(|args| args.into_iter().next()).flatten()
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_extract_result() {
  assert_eq!(
   inner_ty(syn::parse_str::<syn::Type>("Result<String, JsValue>").unwrap()),
   Some(syn::parse_str::<syn::GenericArgument>("String").unwrap())
  );
  assert_eq!(inner_ty(syn::parse_str::<syn::Type>("String").unwrap()), None);
 }
}
