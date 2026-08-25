// This file was procedurally generated from the following sources:
// - src/import-defer/getPrototypeOf.case
// - src/import-defer/ignore/ignore.template
/*---
description: _ [[GetPrototypeOf]] (does not trigger execution)
esid: sec-module-namespace-exotic-objects
features: [import-defer]
flags: [generated, module]
info: |
    [[GetPrototypeOf]] ( )
      1. Return **null**.

---*/


import "./setup_FIXTURE.js";

import defer * as ns from "./dep_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

Object.getPrototypeOf(ns);

assert.sameValue(globalThis.evaluations.length, 0, "It does not trigger evaluation");
