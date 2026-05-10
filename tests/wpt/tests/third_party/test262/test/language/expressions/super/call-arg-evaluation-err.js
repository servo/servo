// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: Returns abrupt completion resulting from ArgumentListEvaluation
info: |
  [...]
  4. Let argList be ArgumentListEvaluation of Arguments.
  5. ReturnIfAbrupt(argList).
features: [class]
---*/

var thrown = new Test262Error();
var thrower = function() {
  throw thrown;
};
var caught;
class C extends Object {
  constructor() {
    try {
      super(thrower());
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
  new C();
} catch (_) {}

assert.sameValue(caught, thrown);
