import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { verbindeKern } from "./kern";
import { t } from "./texte";
import "./styles.css";

document.title = `${t("app.titel")} – ${t("app.untertitel")}`;

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App kern={verbindeKern()} />
  </StrictMode>,
);
