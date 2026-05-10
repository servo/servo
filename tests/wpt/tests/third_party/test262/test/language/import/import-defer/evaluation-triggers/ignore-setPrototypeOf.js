// This file was procedurally generated from the following sources:
// - src/import-defer/setPrototypeOf.case
// - src/import-defer/ignore/ignore.template
/*---
description: _ [[PreventExtensions]] (does not trigger execution)
esid: sec-module-namespace-exotic-objects
features: [import-defer]
flags: [generated, module]
info: |
    [[PreventExtensions]] ( )
      1. Return **true**.

---*/


import "./setup_FIXTURE.js";

import defer * as ns from "./dep_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

Reflect.setPrototypeOf(ns, null);
Reflect.setPrototypeOf(ns, {});

assert.sameValue(globalThis.evaluations.length, 0, "It does not trigger evaluation");
