import turbocharger_init, {
 backend,
 Person,
 wasm_only,
} from "./turbocharger_generated";

(async () => {
 await turbocharger_init();
 let p = wasm_only.new_person();
 p.name = "Bob";
 console.log(p);
 let rowid = await backend.insert_person(p);
 console.log(await backend.get_person(rowid));
})();
