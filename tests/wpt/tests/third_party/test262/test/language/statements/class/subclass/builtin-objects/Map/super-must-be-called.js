// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.1.1
description: Super need to be called to initialize internals
info: |
  23.1.1 The Map Constructor

  ...

  The Map constructor is designed to be subclassable. It may be used as the
  value in an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified Map behaviour must include a super call to the
  Map constructor to create and initialize the subclass instance with the
  internal state necessary to support the Map.prototype built-in methods.
---*/

class M1 extends Map {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new M1();
});

class M2 extends Map {
  constructor() {
    super();
  }
}

new M2();
