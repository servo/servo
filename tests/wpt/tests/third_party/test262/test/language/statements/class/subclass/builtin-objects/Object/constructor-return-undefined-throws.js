// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.2.2
description: Throws a ReferenceError if constructor result is undefined
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
  15. Return envRec.GetThisBinding().

  8.1.1.3.4 GetThisBinding ()

  ...
  3. If envRec.[[thisBindingStatus]] is "uninitialized", throw a ReferenceError
  exception.
  ...

---*/

class Obj extends Object {
  constructor() {
    return undefined;
  }
}

class Obj2 extends Object {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new Obj();
});

assert.throws(ReferenceError, function() {
  new Obj2();
});
