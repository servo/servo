// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.2.1
description: Super need to be called to initialize internals
info: |
  23.2.1 The Set Constructor

  ...

  The Set constructor is designed to be subclassable. It may be used as the
  value in an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified Set behaviour must include a super call to the
  Set constructor to create and initialize the subclass instance with the
  internal state necessary to support the Set.prototype built-in methods.
---*/

class S1 extends Set {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new S1();
});

class S2 extends Set {
  constructor() {
    super();
  }
}

new S2();
