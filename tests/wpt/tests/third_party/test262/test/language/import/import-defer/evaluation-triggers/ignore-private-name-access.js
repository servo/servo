// This file was procedurally generated from the following sources:
// - src/import-defer/private-name-access.case
// - src/import-defer/ignore/ignore.template
/*---
description: PrivateGet and PrivateSet in a namespace object (does not trigger execution)
esid: sec-module-namespace-exotic-objects
features: [import-defer, nonextensible-applies-to-private]
flags: [generated, module]
info: |
    PrivateGet ( O, P )
      1. Let entry be PrivateElementFind(O, P).
      1. If entry is EMPTY, throw a TypeError exception.
      1. If entry.[[Kind]] is either FIELD or METHOD, then
         a. Return entry.[[Value]].
      ...

    PrivateSet ( O, P, value )
      1. Let entry be PrivateElementFind(O, P).
      1. If entry is EMPTY, throw a TypeError exception.
      1. If entry.[[Kind]] is FIELD, then
         a. Set entry.[[Value]] to value.
      ...

---*/


import "./setup_FIXTURE.js";

import defer * as ns from "./dep_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

class Marker extends function (x) { return x } {
  #mark = "bar";

  static mark(obj) {
    new Marker(obj);
  }

  static access(obj) {
    return #mark in obj;
  }
}

assert.throws(TypeError, function () {
  Marker.mark(ns);
});

assert.sameValue(false, Marker.access(ns));

assert.sameValue(globalThis.evaluations.length, 0, "It does not trigger evaluation");
