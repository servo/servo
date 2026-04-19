// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: Abrupt completion from Expression evaluation
info: |
  1. Let propertyNameReference be the result of evaluating Expression.
  2. Let propertyNameValue be ? GetValue(propertyNameReference).

  6.2.3.1 GetValue

  1. ReturnIfAbrupt(V).
---*/

var thrown = new Test262Error();
var caught;
function thrower() {
  throw thrown;
}
var obj = {
  method() {
    try {
      super[thrower()];
    } catch (err) {
      caught = err;
    }
  }
};

obj.method();

assert.sameValue(caught, thrown);
