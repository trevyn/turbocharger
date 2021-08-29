<script>
 import * as backend from "./turbocharger_generated";

 (async () => {
  let person = Object.assign(new backend.Person(), { name: "Bob" });
  let rowid = await backend.insert_person(person);
  console.log("Inserted rowid ", rowid);
 })();
</script>

Hello!<br />

{#await backend.get_person(1n) then person}
 Name: {person.name}
{:catch error}
 Error: {error}
{/await}
