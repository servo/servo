// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.3.1
description: Super need to be called to initialize Boolean internals
info: |
  19.3.1 The Boolean Constructor

  ...
  Subclass constructors that intend to inherit the specified Boolean behaviour
  must include a super call to the Boolean constructor to create and initialize
  the subclass instance with a [[BooleanData]] internal slot.
---*/

class Bln extends Boolean {
  constructor() {}
}

// Boolean internals are not initialized
assert.throws(ReferenceError, function() {
  new Bln(1);
});

class Bln2 extends Boolean {
  constructor() {
    super();
  }
}

var b = new Bln2(1);
assert(b instanceof Boolean);
