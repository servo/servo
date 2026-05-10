// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword-runtime-semantics-evaluation
description: >
    `this` is bound in the constructor of derived classes immediately before running initializers
info: |
    [...]
    6. Let result be ? Construct(func, argList, newTarget).
    [...]
    10. Perform ? thisER.BindThisValue(result).
    11. Perform ? InitializeInstanceFields(result, F).
    [...]
features: [class-fields-public]
---*/


var probeCtorThis;
var thisDuringField;
var thisFromProbe;
var thisDuringCtor;

class Base {
  constructor() {
    assert.throws(ReferenceError, probeCtorThis);
  }
}

var C = class extends Base {
  field = (thisDuringField = this, thisFromProbe = probeCtorThis());
  constructor() {
    probeCtorThis = () => this;
    assert.throws(ReferenceError, probeCtorThis);
    super();
    thisDuringCtor = this;
  }
};

var instance = new C();

assert.sameValue(thisDuringField, instance);
assert.sameValue(thisFromProbe, instance);
assert.sameValue(thisDuringCtor, instance);
