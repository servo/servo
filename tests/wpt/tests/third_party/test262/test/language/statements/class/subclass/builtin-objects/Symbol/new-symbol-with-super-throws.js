// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.4.1
description: Symbol subclass called with the new operator throws on super()
info: |
  19.4.1 The Symbol Constructor

  ...
  The Symbol constructor is not intended to be used with the new operator or to
  be subclassed. It may be used as the value of an extends clause of a class
  definition but a super call to the Symbol constructor will cause an exception.

  19.4.1.1 Symbol ( [ description ] )

  ...
  1. If NewTarget is not undefined, throw a TypeError exception.
features: [Symbol]
---*/

class S1 extends Symbol {}

assert.throws(TypeError, function() {
  new S1();
});

class S2 extends Symbol {
  constructor() {
    super();
  }
}

assert.throws(TypeError, function() {
  new S2();
});

