use turbocharger::prelude::*;

mod app;

fn main() {
	// wasm_logger::init(wasm_logger::Config::default());
	// console_error_panic_hook::set_once();

	#[cfg(any(feature = "wasm", target_arch = "wasm32"))]
	{
		turbocharger::set_socket_url("ws://localhost:8888/turbocharger_socket".into());
		dioxus_web::launch(app);
	}
}

#[frontend]
fn app(cx: Scope) -> Element {
	// let fut = use_future(&cx, (), |_| async move { app::run_test().await });
	// console_log!("{:?}", fut.value());

	render! (
		div {
			style: "text-align: center;",
			h1 { "🌗 Dioxus 🚀" }
			h3 { "Frontend that scales." }
			p { "Dioxus is a portable, performant, and ergonomic framework for building cross-platform user interfaces in Rust." }
		}
	)
}
