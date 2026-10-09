import { Component, Fragment, type ErrorInfo, type ReactNode } from "react";
import { t } from "./texte";

interface Zustand {
  fehler: Error | null;
  versuch: number;
}

/**
 * Catches errors while drawing a view. Without it React removes the whole interface and
 * only the background stays; here the player sees what failed and can go on.
 */
export class Fehlergrenze extends Component<{ children: ReactNode }, Zustand> {
  state: Zustand = { fehler: null, versuch: 0 };

  static getDerivedStateFromError(fehler: Error): Partial<Zustand> {
    return { fehler };
  }

  componentDidCatch(fehler: Error, info: ErrorInfo) {
    console.error("Fehler in der Oberfläche", fehler, info.componentStack);
  }

  render() {
    const { fehler, versuch } = this.state;
    if (!fehler) return <Fragment key={versuch}>{this.props.children}</Fragment>;
    return (
      <main className="absturz" role="alert">
        <h1>{t("absturz.titel")}</h1>
        <p>{t("absturz.text")}</p>
        <pre className="absturz-details">{`${fehler.message}\n${fehler.stack ?? ""}`}</pre>
        <div className="knopfreihe">
          <button
            type="button"
            className="haupt"
            onClick={() => this.setState({ fehler: null, versuch: versuch + 1 })}
          >
            {t("absturz.weiter")}
          </button>
          <button type="button" onClick={() => window.location.reload()}>
            {t("absturz.neu_laden")}
          </button>
        </div>
      </main>
    );
  }
}
