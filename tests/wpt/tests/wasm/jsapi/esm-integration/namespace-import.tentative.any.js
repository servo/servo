// META: global=window,dedicatedworker,jsshell

promise_test(async () => {
  const { getJsNs, getWasmNs } = await import("./resources/namespace-import.wasm");
  const jsNs = await import("./resources/namespace-import-target.js");
  const wasmNs = await import("./resources/namespace-import-target.wasm");

  assert_equals(getJsNs(), jsNs, "JS module namespace");
  assert_equals(getWasmNs(), wasmNs, "WebAssembly module namespace");
}, "Import a re-exported module namespace into WebAssembly");
