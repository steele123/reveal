import "./styles.css";
import App from "./reveal.svelte";
import { logFrontendError, logFrontendInfo } from "$lib/logging";

window.addEventListener("error", (event) => {
  logFrontendError("Uncaught frontend error", event.error ?? event.message);
});

window.addEventListener("unhandledrejection", (event) => {
  logFrontendError("Unhandled frontend promise rejection", event.reason);
});

logFrontendInfo("Frontend bootstrap started");

const app = new App({
  target: document.getElementById("app")!,
});

logFrontendInfo("Frontend mounted successfully");

export default app;
