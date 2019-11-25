"use strict";
self.importScripts("/resources/testharness.js");
self.importScripts("./create-empty-wasm-module.js");

let state = "start in worker";

self.onmessage = e => {
  if (e.data === "start in window") {
    assert_equals(state, "start in worker");
    e.source.postMessage(state);
    state = "waiting for message from the window";
  } else if (e.data === "we are expecting a messageerror due to the worker sending us a WebAssembly.Module")  {
    assert_equals(state, "waiting for message from the window");
    e.source.postMessage(createEmptyWasmModule());
    state = "done in worker";
  } else {
    e.source.postMessage(`worker onmessage was reached when in state "${state}" and data ${e.data}`);
  }
};