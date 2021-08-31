Full Turbocharger template with Svelte, Tailwind, and Turbosql.

Run full stack:

```
npm start
```

Run full stack in watch mode, will reload on save of any frontend or backend file:

```
cargo install cargo-watch
npm run watch
```

Run Svelte frontend only in Hot Module Replacement dev mode (you'll need to manually run the server with `npm start` in a separate terminal, and re-run it if you change any Rust code):

```
npm run hmr
```

make release build:

```
npm run build
```
