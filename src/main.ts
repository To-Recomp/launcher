import { mount } from "svelte";
import "@fontsource-variable/inter";
import "./app.css";
import App from "./App.svelte";
import { startLanguage } from "./lib/i18n/start";

// The language is known before the first screen, so nothing flashes in the wrong one.
startLanguage().then(() => mount(App, { target: document.getElementById("app")! }));
