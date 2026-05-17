// META: global=window,dedicatedworker,jsshell

promise_test(async (t) => {
  await promise_rejects_js(
    t,
    SyntaxError,
    import("./resources/resolve-export.js")
  );
}, "ResolveExport on invalid re-export from WebAssembly");
