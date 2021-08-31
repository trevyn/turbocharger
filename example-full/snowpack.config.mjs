/** @type {import("snowpack").SnowpackUserConfig } */
export default {
 mount: {
  "./src-frontend": {
   url: "/",
  },
 },
 plugins: [
  [
   "@emily-curry/snowpack-plugin-wasm-pack",
   {
    projectPath: ".",
    outDir: "src-frontend/turbocharger_generated",
   },
  ],
  "@snowpack/plugin-svelte",
  "@snowpack/plugin-postcss",
 ],
 exclude: ["**/*.json", "**/*.md"],
 optimize: {
  bundle: true,
  target: "es2020",
 },
 devOptions: {
  tailwindConfig: "./tailwind.config.js",
  port: 8081,
 },
};
