import turbocharger_init, * as app from "./turbocharger_generated";

(async () => {
 await turbocharger_init();
 if (
  window.location.href.startsWith("http://127.0.0.1:3000/") ||
  window.location.href.startsWith("http://localhost:3000/")
 )
  app.set_socket_url("ws://127.0.0.1:8080/turbocharger_socket");
 app.start_web();
})();
