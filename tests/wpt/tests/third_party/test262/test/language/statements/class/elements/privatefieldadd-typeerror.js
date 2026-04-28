// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Class construction should error if attempting to add private field twice
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
info: |
  [[Construct]] ( argumentsList, newTarget)
    ...
    8. If kind is "base", then
      a. Perform OrdinaryCallBindThis(F, calleeContext, thisArgument).
      b. Let result be InitializeInstanceFields(thisArgument, F).
      c. If result is an abrupt completion, then
        i. Remove calleeContext from execution context stack and restore callerContext as the running execution context.
        ii. Return Completion(result).

  InitializeInstanceFields ( O, constructor )
    1. Assert: Type ( O ) is Object.
    2. Assert: Assert constructor is an ECMAScript function object.
    3. Let fieldRecords be the value of constructor's [[Fields]] internal slot.
    4. For each item fieldRecord in order from fieldRecords,
      a. Perform ? DefineField(O, fieldRecord).
    5. Return.

  DefineField(receiver, fieldRecord)
    ...
    8. If fieldName is a Private Name,
      a. Perform ? PrivateFieldAdd(fieldName, receiver, initValue).

  PrivateFieldAdd (P, O, value)
    1. Assert: P is a Private Name value.
    2. If O is not an object, throw a TypeError exception.
    3. Let entry be PrivateFieldFind(P, O).
    4. If entry is not empty, throw a TypeError exception.
    ...

features: [class, class-fields-private]
---*/


class A {
  constructor(arg) {
    return arg;
  }
}

class C extends A {
  #x;

  constructor(arg) {
    super(arg);
  }
}

var containsprivatex = new C();

assert.throws(TypeError, function() {
  // After the super call in C's constructor, the `this` value in C will
  // already have "#x" in it's [[PrivateFieldValues]]
  new C(containsprivatex);
})
