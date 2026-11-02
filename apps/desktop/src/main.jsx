import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App.jsx";
import { LiveApp } from "./LiveApp.jsx";
import "./styles.css";

if (window.__TAURI__?.core?.invoke) document.title = "Software Factory · Local alpha";

createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    {window.__TAURI__?.core?.invoke ? <LiveApp /> : <App />}
  </React.StrictMode>,
);
