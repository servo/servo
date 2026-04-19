// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-module-namespace-exotic-objects-get-p-receiver-EnsureDeferredNamespaceEvaluation
description: >
  Host resolution errors are reported eagerly
info: |
  LoadRequestedModules ([ _hostDefined_ ])
    - just notice that it does not check if the module is deferred

flags: [async, module]
includes: [asyncHelpers.js]
features: [import-defer]
---*/

asyncTest(async () => {
  globalThis.evaluated = false;
  try {
    await import("./main_FIXTURE.js");
  } catch {
    assert.sameValue(globalThis.evaluated, false, "The module should not be evaluated");
    return;
  }
  throw new Error("The module should throw");
})
