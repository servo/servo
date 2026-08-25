// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class Base {
  m() { return "pass"; }
  static m() { return "fail"; }
}

var key = {
  toString() {
    return "computed";
  }
};

let obj1 = new class extends Base {
  [key]() {}

  // Private method with a directly preceding method using a computed key led
  // to assigning the wrong home object.
  #m() { return super.m(); }
  m() { return this.#m(); }
};

assert.sameValue(obj1.m(), "pass");

let obj2 = new class extends Base {
  // Same test as above, but this time preceded by a static method.
  static [key]() {}

  #m() { return super.m(); }
  m() { return this.#m(); }
};

assert.sameValue(obj2.m(), "pass");

