// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 22.1.1
description: Super need to be called to initialize internals
info: |
  22.1.1 The Array Constructor

  ...

  The Array constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition. Subclass constructors that
  intend to inherit the exotic Array behaviour must include a super call to the
  Array constructor to initialize subclass instances that are exotic Array
  objects.
---*/

class A extends Array {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new A();
});

class A2 extends Array {
  constructor() {
    super();
  }
}

new A2();
