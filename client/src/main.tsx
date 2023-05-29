import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import Messenger from "./Messenger/Messenger";
import ThemeProviderWrapper from "./theme/ThemeProvider";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProviderWrapper>
      <Messenger />
    </ThemeProviderWrapper>
  </React.StrictMode>
);
