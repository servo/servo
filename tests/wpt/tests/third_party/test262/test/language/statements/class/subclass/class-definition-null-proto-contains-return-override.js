// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
es6id: 14.5.14
description: >
    The constructor of a null-extending class can contain an explicit return value.
info: |
  Runtime Semantics: ClassDefinitionEvaluation

  [...]
  15. If ClassHeritageopt is present, then set F's [[ConstructorKind]] internal slot to "derived".
  [...]

  9.2.2 [[Construct]]

  [...]
  13. If result.[[Type]] is return, then
     a. If Type(result.[[Value]]) is Object, return NormalCompletion(result.[[Value]]).
  [...]
---*/
var obj;

class Foo extends null {
  constructor() {
    return obj = {};
  }
}

var f = new Foo();

assert.sameValue(f, obj);
assert.sameValue(Object.getPrototypeOf(f), Object.prototype);
