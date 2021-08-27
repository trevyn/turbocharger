/** @type {import("snowpack").SnowpackUserConfig } */
export default {
 mount: {
  "./src": {
   url: "/",
  },
 },
 plugins: [
  [
   "@emily-curry/snowpack-plugin-wasm-pack",
   {
    projectPath: "..",
    outDir: "frontend/src/turbocharger_generated",
   },
  ],
 ],
 optimize: {
  bundle: true,
  target: "es2020",
 },
 devOptions: {
  port: 8081,
 },
};
