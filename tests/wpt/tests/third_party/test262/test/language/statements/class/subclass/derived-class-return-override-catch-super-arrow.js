// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
description: >
  TypeError from `return 0` is not catchable with `super` called in catch block
  from an arrow function.
---*/

class C extends class {} {
  constructor() {
    var f = () => super();

    try {
      return 0;
    } catch(e) {
      f();
    }
  }
}

assert.throws(TypeError, function() {
  new C();
});
