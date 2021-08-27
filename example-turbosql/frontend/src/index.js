import turbocharger_init, * as backend from "./turbocharger_generated";

(async () => {
 await turbocharger_init();
 let p = new backend.Person();
 p.name = "Bob";
 let rowid = await backend.insert_person(p);

 let row = await backend.get_person(BigInt(rowid));
 console.log(row.rowid, row.name);
})();
