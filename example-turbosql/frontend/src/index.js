import turbocharger_init, * as backend from "./turbocharger_generated";

(async () => {
 await turbocharger_init();
 let person = Object.assign(new backend.Person(), { name: "Bob" });
 let rowid = await backend.insert_person(person);
 console.log((await backend.get_person(rowid)).toJSON());
})();
