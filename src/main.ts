import { mount } from "svelte";
import "@fontsource-variable/inter";
import "./app.css";
import App from "./App.svelte";
import { startLanguage } from "./lib/i18n/start";

startLanguage().then(() => mount(App, { target: document.getElementById("app")! }));
