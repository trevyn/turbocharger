import wasminit, * as wasm from "../../pkg";

function append(t) {
 document.body.appendChild(document.createTextNode(t));
}

async function main() {
 append("Hello from JS.");
 await wasminit("dist/example-minimal/index_bg.wasm");
 append(await wasm.get_wasm_greeting());
 // append("Hello from backend.");
}

main();
