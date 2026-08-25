// This file was procedurally generated from the following sources:
// - src/import-defer/isExtensible.case
// - src/import-defer/ignore/ignore.template
/*---
description: _ [[IsExtensible]] (does not trigger execution)
esid: sec-module-namespace-exotic-objects
features: [import-defer]
flags: [generated, module]
info: |
    [[IsExtensible]] ( )
      1. Return **false**.

---*/


import "./setup_FIXTURE.js";

import defer * as ns from "./dep_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

Object.isExtensible(ns);

assert.sameValue(globalThis.evaluations.length, 0, "It does not trigger evaluation");
