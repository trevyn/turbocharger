fn generic_type_with_ident<'a>(ty: &'a syn::Type, ident: &str) -> Option<&'a syn::Type> {
 match ty {
  syn::Type::Path(syn::TypePath { path, .. }) => generic_path_with_ident(path, ident),
  _ => None,
 }
}

fn generic_path_with_ident<'a>(path: &'a syn::Path, ident: &str) -> Option<&'a syn::Type> {
 let pathsegment = path.segments.last()?;
 if pathsegment.ident != ident {
  return None;
 }
 let args = match &pathsegment.arguments {
  syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments { args, .. }) => args,
  _ => return None,
 };
 match &args.first()? {
  syn::GenericArgument::Type(ty) => Some(ty),
  syn::GenericArgument::Binding(syn::Binding { ty, .. }) => Some(ty),
  _ => None,
 }
}

pub fn extract_stream(ty: &syn::Type) -> Option<&syn::Type> {
 let bounds = match ty {
  syn::Type::ImplTrait(syn::TypeImplTrait { bounds, .. }) => bounds,
  syn::Type::Path(_) => {
   match generic_type_with_ident(generic_type_with_ident(ty, "Pin")?, "Box")? {
    syn::Type::TraitObject(syn::TypeTraitObject { bounds, .. }) => bounds,
    _ => return None,
   }
  }
  _ => return None,
 };

 let path = match &bounds.first()? {
  syn::TypeParamBound::Trait(syn::TraitBound { path, .. }) => path,
  _ => return None,
 };

 generic_path_with_ident(path, "Stream")
}

pub fn extract_result(ty: &syn::Type) -> Option<&syn::Type> {
 generic_type_with_ident(ty, "Result")
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_extract() {
  assert_eq!(
   extract_result(&syn::parse_str::<syn::Type>("Result<String, JsValue>").unwrap()),
   Some(&syn::parse_str::<syn::Type>("String").unwrap())
  );
  assert_eq!(
   extract_result(&syn::parse_str::<syn::Type>("foo::Result<String>").unwrap()),
   Some(&syn::parse_str::<syn::Type>("String").unwrap())
  );
  assert_eq!(
   extract_result(&syn::parse_str::<syn::Type>("Result<Vec<u8>, tracked::StringError>").unwrap()),
   Some(&syn::parse_str::<syn::Type>("Vec<u8>").unwrap())
  );

  assert_eq!(extract_result(&syn::parse_str::<syn::Type>("String").unwrap()), None);

  assert_eq!(
   extract_stream(&syn::parse_str::<syn::Type>("impl Stream<Item = u32>").unwrap()),
   Some(&syn::parse_str::<syn::Type>("u32").unwrap())
  );
  assert_eq!(
   extract_stream(&syn::parse_str::<syn::Type>("impl futures::Stream<Item = u32>").unwrap()),
   Some(&syn::parse_str::<syn::Type>("u32").unwrap())
  );
  assert_eq!(
   extract_stream(
    &syn::parse_str::<syn::Type>("impl turbocharger_thing::Stream<Item = u32>").unwrap()
   ),
   Some(&syn::parse_str::<syn::Type>("u32").unwrap())
  );
  assert_eq!(
   extract_stream(
    &syn::parse_str::<syn::Type>("foo::Pin<wee::Box<dyn turbocharger_thing::Stream<Item = u32>>>")
     .unwrap()
   ),
   Some(&syn::parse_str::<syn::Type>("u32").unwrap())
  );
  assert_eq!(
   extract_stream(
    &syn::parse_str::<syn::Type>("impl Stream<Item = Result<i32, anyhow::Error>>").unwrap()
   ),
   Some(&syn::parse_str::<syn::Type>("Result<i32, anyhow::Error>").unwrap())
  );
  assert_eq!(
   extract_stream(&syn::parse_str::<syn::Type>("impl Stream<Item = anyhow::Result<i32>>").unwrap()),
   Some(&syn::parse_str::<syn::Type>("anyhow::Result<i32>").unwrap())
  );
  assert_eq!(
   extract_stream(
    &syn::parse_str::<syn::Type>(
     "Pin<Box<dyn Stream<Item = Result<Vec<u8>, tracked::StringError>>>>"
    )
    .unwrap()
   ),
   Some(&syn::parse_str::<syn::Type>("Result<Vec<u8>, tracked::StringError>").unwrap())
  );

  assert_eq!(extract_stream(&syn::parse_str::<syn::Type>("u32").unwrap()), None);
 }
}
