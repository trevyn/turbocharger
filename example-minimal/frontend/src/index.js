import turbocharger_init, * as tc from "./turbocharger_generated";

function append(t) {
 document.body.appendChild(document.createTextNode(t));
}

async function main() {
 append("Hello from JS.");
 await turbocharger_init("turbocharger_generated/index_bg.wasm");
 append(await tc.wasm_get_greeting());
 append(await tc.backend_get_greeting());
}

main();
