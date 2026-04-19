// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 24.1.2
description: Super need to be called to initialize internals
info: |
  24.1.2 The ArrayBuffer Constructor

  ...

  The ArrayBuffer constructor is designed to be subclassable. It may be used as
  the value of an extends clause of a class definition. Subclass constructors
  that intend to inherit the specified ArrayBuffer behaviour must include a
  super call to the ArrayBuffer constructor to create and initialize subclass
  instances with the internal state necessary to support the
  ArrayBuffer.prototype built-in methods.
---*/

class AB1 extends ArrayBuffer {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new AB1(1);
});

class AB2 extends ArrayBuffer {
  constructor(length) {
    super(length);
  }
}

new AB2(1);
