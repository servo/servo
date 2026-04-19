// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.3
description: Super need to be called to initialize internals
info: |
  25.4.3 The Promise Constructor

  ...

  The Promise constructor is designed to be subclassable. It may be used as the
  value in an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified Promise behaviour must include a super call
  to the Promise constructor to create and initialize the subclass instance with
  the internal state necessary to support the Promise and Promise.prototype
  built-in methods.
---*/

class Prom1 extends Promise {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new Prom1();
});

class Prom2 extends Promise {
  constructor(exec) {
    super(exec);
  }
}

new Prom2(function() {});
