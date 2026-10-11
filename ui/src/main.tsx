import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { Fehlergrenze } from "./Fehlergrenze";
import { verbindeKern } from "./kern";
import { sortierbareTabellen } from "./sortieren";
import { t } from "./texte";
import "./styles.css";

document.title = `${t("app.titel")} – ${t("app.untertitel")}`;
sortierbareTabellen();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Fehlergrenze>
      <App kern={verbindeKern()} />
    </Fehlergrenze>
  </StrictMode>,
);
