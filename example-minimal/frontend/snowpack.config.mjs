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
