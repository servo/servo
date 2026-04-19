// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 20.3.2
description: Super need to be called to initialize internals
info: |
  20.3.2 The Date Constructor

  ...

  The Date constructor is a single function whose behaviour is overloaded based
  upon the number and types of its arguments.

  The Date constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified Date behaviour must include a super call to
  the Date constructor to create and initialize the subclass instance with a
  [[DateValue]] internal slot.
---*/

class D extends Date {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new D(0);
});

class D2 extends Date {
  constructor(d) {
    super(d);
  }
}

new D2(0);
