import React from "react";
import ReactDOM from "react-dom/client";

import "@fontsource/barlow/400.css";
import "@fontsource/barlow/600.css";
import "@fontsource/barlow/700.css";
import "@fontsource/barlow-condensed/500.css";
import "@fontsource/barlow-condensed/600.css";
import "@fontsource/barlow-condensed/700.css";
import "@fontsource/jetbrains-mono/400.css";
import "@fontsource/jetbrains-mono/500.css";
import "../../design/tokens.css";
import "../../design/components/bundle.css";
import "./app.css";

// O bundle do design system é um IIFE que lê window.React e publica window.Tank.
(window as unknown as { React: typeof React }).React = React;
await import("../../design/components/bundle.js");
const { App } = await import("./App");

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
