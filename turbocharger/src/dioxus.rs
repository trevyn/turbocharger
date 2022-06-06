use dioxus::prelude::*;
use futures_util::{Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;

#[macro_export]
macro_rules! to_owned {
    ($($es:ident),+) => {$(
        let $es = $es.to_owned();
    )*}
}

pub enum Poll<T> {
 Pending,
 Ready(T),
}

pub use Poll::*;

pub fn use_stream<T>(
 cx: &ScopeState,
 stream: impl FnOnce() -> Pin<Box<(dyn Stream<Item = T>)>> + 'static,
) -> &UseState<Poll<T>> {
 use_stream_map(cx, stream, std::convert::identity)
}

pub fn use_stream_map<'a, T, U>(
 cx: &'a ScopeState,
 stream: impl FnOnce() -> Pin<Box<(dyn Stream<Item = T>)>> + 'static,
 map_fn: impl Fn(T) -> U + 'static,
) -> &'a UseState<Poll<U>> {
 let data = use_state(cx, || Pending);
 let data_cloned = data.clone();
 let _: &'a CoroutineHandle<()> = use_coroutine(cx, |_| async move {
  let mut conn = stream();
  while let Some(r) = conn.next().await {
   data_cloned.set(Ready(map_fn(r)));
  }
 });
 data
}

pub fn use_backend<T>(
 cx: &ScopeState,
 fut: impl FnOnce() -> Pin<Box<(dyn Future<Output = T>)>> + 'static,
) -> &UseState<Poll<T>> {
 use_backend_map(cx, fut, std::convert::identity)
}

pub fn use_backend_map<'a, T, U>(
 cx: &'a ScopeState,
 fut: impl FnOnce() -> Pin<Box<(dyn Future<Output = T>)>> + 'static,
 map_fn: impl Fn(T) -> U + 'static,
) -> &'a UseState<Poll<U>> {
 let data = use_state(cx, || Pending);
 let data_cloned = data.clone();
 let _: &'a CoroutineHandle<()> = use_coroutine(cx, |_| async move {
  data_cloned.set(Ready(map_fn(fut().await)));
 });
 data
}
