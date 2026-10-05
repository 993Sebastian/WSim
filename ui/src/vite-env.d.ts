/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** "web": the simulation core runs as WebAssembly in the browser (build mode web). */
  readonly VITE_KERN?: string;
}

declare module "*.yaml" {
  const daten: Record<string, unknown>;
  export default daten;
}
