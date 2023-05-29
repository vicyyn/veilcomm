import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import App from "./App";
import ThemeProviderWrapper from "./theme/ThemeProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <ThemeProviderWrapper>
    <App />
  </ThemeProviderWrapper>
);
