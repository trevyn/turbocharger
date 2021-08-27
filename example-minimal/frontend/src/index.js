import turbocharger_init, {
 wasm_only as wasm,
 backend,
} from "./turbocharger_generated";

function append(t) {
 document.body.appendChild(document.createTextNode(t));
 document.body.appendChild(document.createElement("br"));
}

(async () => {
 append("Hello from JS.");
 await turbocharger_init("turbocharger_generated/index_bg.wasm");
 append(await wasm.get_local_greeting1());
 append(await wasm.get_local_greeting2());
 append(await backend.get_backend_test());
 await backend.get_backend_test_no_retval();
 append(await backend.get_backend_test_with_delay());
 append(await backend.get_backend_test_with_string("human"));
 append(await backend.get_backend_test_with_i64_i32(1n, 2));
 for (let x = 0; x < 50; x++)
  backend.get_backend_test_with_delay().then((r) => {
   append(r);
  });
})();
