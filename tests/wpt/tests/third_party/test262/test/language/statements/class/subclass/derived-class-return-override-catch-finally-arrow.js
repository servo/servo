// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
description: >
  `super()` in finally block is executed before checking for missing `super()`
  call when `return` is in a catch block. The `super()` call is performed
  through an arrow function.
---*/

class C extends class {} {
  constructor() {
    var f = () => super();

    try {
      throw null;
    } catch(e) {
      return;
    } finally {
      f();
    }
  }
}

var o = new C();
assert.sameValue(typeof o, "object");
