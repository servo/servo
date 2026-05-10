// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword-runtime-semantics-evaluation
description: >
    when calling `super()` for a second time in a derived class, the super constructor is run twice but the field initializers are only run once
info: |
    [...]
    6. Let result be ? Construct(func, argList, newTarget).
    [...]
    10. Perform ? thisER.BindThisValue(result).
    11. Perform ? InitializeInstanceFields(result, F).
    [...]
features: [class-fields-public]
---*/


var baseCtorCalled = 0;
var fieldInitCalled = 0;
class Base {
  constructor() {
    ++baseCtorCalled;
  }
}

var C = class extends Base {
  field = ++fieldInitCalled;
  constructor() {
    assert.sameValue(baseCtorCalled, 0);
    assert.sameValue(fieldInitCalled, 0);
    super();
    assert.sameValue(baseCtorCalled, 1);
    assert.sameValue(fieldInitCalled, 1);
    assert.throws(ReferenceError, () => super());
  }
};

new C();

assert.sameValue(baseCtorCalled, 2);
assert.sameValue(fieldInitCalled, 1);
