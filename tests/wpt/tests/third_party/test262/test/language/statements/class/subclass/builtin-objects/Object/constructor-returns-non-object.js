// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.2.2
description: The Type of the return value must be an Object
info: |
  9.2.2 [[Construct]] ( argumentsList, newTarget)

  ...
  11. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
  ...
  13. If result.[[type]] is return, then
    a. If Type(result.[[value]]) is Object, return
    NormalCompletion(result.[[value]]).
    ...
    c. If result.[[value]] is not undefined, throw a TypeError exception.
  ...

  6.1.7.2 Object Internal Methods and Internal Slots

  ...
  If any specified use of an internal method of an exotic object is not
  supported by an implementation, that usage must throw a TypeError exception
  when attempted.

  6.1.7.3 Invariants of the Essential Internal Methods

  [[Construct]] ( )
    - The Type of the return value must be Object.
---*/

class Obj extends Object {
  constructor() {
    return 42;
  }
}

assert.throws(TypeError, function() {
  var obj = new Obj();
});
