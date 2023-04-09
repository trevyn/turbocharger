import { expect } from "@esm-bundle/chai";
import turbocharger_init, * as backend from "../../dist/assets/dioxus/turbocharger-tests.js";

it("does stuff", async function () {
	this.timeout(10000);
	await turbocharger_init(
		"../../dist/assets/dioxus/turbocharger-tests_bg.wasm"
	);
	expect(await backend.run_test()).to.equal(42);
	expect(await backend.one_hundred()).to.equal(100);
	expect(await backend.two_hundred()).to.equal(200);
	expect(await backend.two_hundred_increment()).to.equal(201);

	// backend.set_socket_url("ws://localhost:8080/turbocharger_socket");
	// let person = Object.assign(new backend.Person(), { name: "Bob" });
	// let rowid = await backend.insert_person(person);
	// console.log("Inserted rowid ", rowid.toString());
});
