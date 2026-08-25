// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: >
  Behavior when invocation of "parent" constructor returns an abrupt completion
info: |
  [...]
  6. Let result be ? Construct(func, argList, newTarget).
features: [class]
---*/

var thrown = new Test262Error();
var caught;
function Parent() {
  throw thrown;
}

class Child extends Parent {
  constructor() {
    try {
      super();
    } catch (err) {
      caught = err;
    }
  }
}

// When the "construct" invocation completes and the "this" value is
// uninitialized, the specification dictates that a ReferenceError must be
// thrown. That behavior is tested elsewhere, so the error is ignored (if it is
// produced at all).
try {
  new Child();
} catch (_) {}

assert.sameValue(caught, thrown);
