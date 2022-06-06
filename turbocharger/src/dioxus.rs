use dioxus::prelude::*;
use futures_util::{Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;

pub fn use_stream<'a, T, U>(
 cx: &'a ScopeState,
 stream: impl FnOnce() -> Pin<Box<(dyn Stream<Item = T>)>> + 'static,
 map_fn: impl Fn(T) -> U + 'static,
) -> &'a UseState<Option<U>> {
 let data = use_state(cx, || None);
 let data_cloned = data.clone();
 let _: &'a CoroutineHandle<()> = use_coroutine(cx, |_| async move {
  let mut conn = stream();
  while let Some(r) = conn.next().await {
   data_cloned.set(Some(map_fn(r)));
  }
 });
 data
}

pub fn use_backend<'a, T>(
 cx: &'a ScopeState,
 fut: impl FnOnce() -> Pin<Box<(dyn Future<Output = T>)>> + 'static,
) -> &'a UseState<Option<T>> {
 let data = use_state(cx, || None);
 let data_cloned = data.clone();
 let _: &'a CoroutineHandle<()> = use_coroutine(cx, |_| async move {
  data_cloned.set(Some(fut().await));
 });
 data
}
