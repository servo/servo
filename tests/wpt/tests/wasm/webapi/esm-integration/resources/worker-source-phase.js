import source modSource from "./worker.wasm";
import { pm } from "./worker-helper.js";

pm.checks = [
  modSource instanceof WebAssembly.Module,
  (await import.source('./worker.wasm') === modSource)
];

await WebAssembly.instantiate(modSource, {
  "./worker-helper.js": {
    "pm": pm
  }
});