// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.1
description: >
  super must be called to initialize Function internal slots
info: |
  19.2.1 The Function Constructor

  ...

  The Function constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition.  Subclass constructors that
  intend to inherit the specified Function behaviour must include a super call
  to the Function constructor to create and initialize a subclass instances with
  the internal slots necessary for built-in function behaviour.
  ...
---*/

class Fn extends Function {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new Fn();
});

class Fn2 extends Function {
  constructor() {
    super();
  }
}

var fn = new Fn2();
assert(fn instanceof Function);
