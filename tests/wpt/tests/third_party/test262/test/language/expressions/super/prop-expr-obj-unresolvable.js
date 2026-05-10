// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: Abrupt completion from Reference resolution
info: |
  1. Let propertyNameReference be the result of evaluating Expression.
  2. Let propertyNameValue be ? GetValue(propertyNameReference).

  6.2.3.1 GetValue

  1. ReturnIfAbrupt(V).
  2. If Type(V) is not Reference, return V.
  3. Let base be GetBase(V).
  4. If IsUnresolvableReference(V) is true, throw a ReferenceError exception.
---*/

var caught;
var obj = {
  method() {
    try {
      super[test262unresolvable];
    } catch (err) {
      caught = err;
    }
  }
};

obj.method();

assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, ReferenceError);
