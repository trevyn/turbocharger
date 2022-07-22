import { expect } from "@esm-bundle/chai";
import turbocharger_init, * as backend from "../../dist/assets/dioxus/turbocharger-tests.js";

it("does stuff", async () => {
 await turbocharger_init("../../dist/assets/dioxus/turbocharger-tests_bg.wasm");
 expect(await backend.run_test()).to.equal(42);

 // backend.set_socket_url("ws://localhost:8080/turbocharger_socket");
 // let person = Object.assign(new backend.Person(), { name: "Bob" });
 // let rowid = await backend.insert_person(person);
 // console.log("Inserted rowid ", rowid.toString());
});
