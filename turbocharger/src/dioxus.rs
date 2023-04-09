use dioxus::prelude::*;

pub fn use_stream<C, T, S>(
	cx: &ScopeState,
	stream: impl FnOnce() -> S + 'static,
	callback: impl Fn(&mut C, T) + 'static,
) -> &UseRef<C>
where
	C: Default + 'static,
	S: futures_util::Stream<Item = T>,
{
	let state = use_ref(cx, <C as Default>::default);

	let state_cloned = state.clone();
	let _ = use_future(cx, (), |_| async move {
		let stream = stream();
		futures_util::pin_mut!(stream);

		while let Some(value) = futures_util::StreamExt::next(&mut stream).await {
			let mut container = state_cloned.write();
			callback(&mut *container, value);
			state_cloned.needs_update();
		}
	});

	state
}

#[allow(dead_code)]
fn test_compile_use_stream(cx: Scope) {
	fn make_stream() -> impl futures_util::Stream<Item = Vec<u8>> {
		futures_channel::mpsc::unbounded().1
	}
	let _latest = use_stream(cx, make_stream, |s, v| *s = Some(v));
	let _vec = use_stream(cx, make_stream, |s: &mut Vec<_>, v| s.push(v));
}
