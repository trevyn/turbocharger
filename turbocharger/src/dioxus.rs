use dioxus::prelude::*;

pub fn use_stream<R, S, T, U>(
 cx: &ScopeState,
 stream: impl FnOnce() -> S + 'static,
 initial: impl FnOnce() -> U,
 cb: impl Fn(&UseState<U>, Option<T>) -> R + 'static,
) -> &UseState<U>
where
 S: futures_util::Stream<Item = T>,
{
 let state = use_state(cx, initial);
 let state_cloned = state.clone();
 let _: &CoroutineHandle<()> = use_coroutine(cx, |_| async move {
  let stream = stream();
  futures_util::pin_mut!(stream);
  while let Some(val) = futures_util::StreamExt::next(&mut stream).await {
   cb(&state_cloned, Some(val));
  }
  cb(&state_cloned, None);
 });
 state
}

// let _ = use_stream(&cx, (), |_| make_stream(), || None, |s, v| v.map(|v| s.set(Some(v))));
// let _ = use_stream(&cx, (), |_| make_stream(), Vec::new, |s, v| v.map(|v| s.make_mut().push(v)));
