// Access to the simulation core. In the desktop app via Tauri IPC, in the browser
// preview (development, UI tests) a stand-in without game logic.
import { invoke, isTauri } from "@tauri-apps/api/core";

export interface KernInfo {
  version: string;
}

export interface Kern {
  /** False for the browser preview without simulation core. */
  readonly echt: boolean;
  info(): Promise<KernInfo>;
}

const tauriKern: Kern = {
  echt: true,
  info: () => invoke<KernInfo>("kern_info"),
};

const vorschauKern: Kern = {
  echt: false,
  info: async () => ({ version: "vorschau" }),
};

export function verbindeKern(): Kern {
  return isTauri() ? tauriKern : vorschauKern;
}
