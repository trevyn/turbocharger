import turbocharger_init, {
 wasm_only as wasm,
 backend,
} from "./turbocharger_generated";

function append(t) {
 document.body.appendChild(document.createTextNode(t));
}

async function main() {
 append("Hello from JS.");
 await turbocharger_init("turbocharger_generated/index_bg.wasm");
 append(await wasm.get_local_greeting1());
 append(await wasm.get_local_greeting2());
 append(await backend.get_backend_test());
 append(await backend.get_remote_greeting("bobert"));
 append(await backend.get_backend_test_with_string("flaubert"));
 append(await backend.get_remote_greeting());
 append(await backend.get_remote_greeting());
 append(await backend.get_remote_greeting());
 append(await backend.get_remote_greeting());
 append(await backend.get_remote_greeting());
 for (let x = 0; x < 100; x++)
  backend.get_remote_greeting().then((r) => {
   append(r);
  });
}

main();
