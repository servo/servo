// This file was procedurally generated from the following sources:
// - src/import-defer/set-string-exported.case
// - src/import-defer/ignore/ignore.template
/*---
description: _ [[Set]] of a string which is an export name (does not trigger execution)
esid: sec-module-namespace-exotic-objects
features: [import-defer]
flags: [generated, module]
info: |
    [[Set]] ( _P_, _V_, _Receiver_ )
      1. Return **false**.

---*/


import "./setup_FIXTURE.js";

import defer * as ns from "./dep_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

try {
  ns.exported = "hi";
} catch (_) {}

assert.sameValue(globalThis.evaluations.length, 0, "It does not trigger evaluation");
