// This file was procedurally generated from the following sources:
// - src/import-defer/ownPropertyKeys-symbols.case
// - src/import-defer/trigger/trigger.template
/*---
description: _ [[OwnPropertyKeys]] (triggers execution)
esid: sec-module-namespace-exotic-objects
features: [import-defer]
flags: [generated, module]
info: |
    [[OwnPropertyKeys]] ( )
      1. Let _exports_ be ? GetModuleExportsList(_O_).
      1. ...

---*/


import "./setup_FIXTURE.js";

import defer * as ns from "./dep_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

Object.getOwnPropertySymbols(ns);

assert(globalThis.evaluations.length > 0, "It triggers evaluation");
