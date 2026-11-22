import { mount } from "svelte";
import "@fontsource-variable/inter";
import "./app.css";
import App from "./App.svelte";
import { startLanguage } from "./lib/i18n/start";
import { appearance } from "./lib/launcher/appearance.svelte";

// Language and theme are known before the first screen, so nothing flashes in the wrong one.
Promise.all([startLanguage(), appearance.load()]).then(() => mount(App, { target: document.getElementById("app")! }));
