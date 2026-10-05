// Builds the simulation core as WebAssembly (crate wsim-web) and puts it next to the
// page of the browser version (public/wsim_web.wasm).
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ui = join(dirname(fileURLToPath(import.meta.url)), "..");
const wurzel = join(ui, "..");
execFileSync(
  "cargo",
  ["build", "-p", "wsim-web", "--target", "wasm32-unknown-unknown", "--release"],
  { cwd: wurzel, stdio: "inherit" },
);
mkdirSync(join(ui, "public"), { recursive: true });
copyFileSync(
  join(wurzel, "target/wasm32-unknown-unknown/release/wsim_web.wasm"),
  join(ui, "public/wsim_web.wasm"),
);
console.log("public/wsim_web.wasm geschrieben");
