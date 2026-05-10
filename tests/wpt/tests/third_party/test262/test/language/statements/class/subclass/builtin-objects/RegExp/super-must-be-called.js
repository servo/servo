// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.3
description: Super need to be called to initialize internals
info: |
  21.2.3 The RegExp Constructor

  ...

  The RegExp constructor is designed to be subclassable. It may be used as the
  value of an extends clause of a class definition. Subclass constructors that
  intend to inherit the specified RegExp behaviour must include a super call to
  the RegExp constructor to create and initialize subclass instances with the
  necessary internal slots.
---*/

class RE1 extends RegExp {
  constructor() {}
}

assert.throws(ReferenceError, function() {
  new RE1();
});

class RE2 extends RegExp {
  constructor() {
    super();
  }
}

new RE2();
