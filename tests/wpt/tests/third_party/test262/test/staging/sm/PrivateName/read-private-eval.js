// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

class A {
  #x = 14;
  g() {
    return eval('this.#x');
  }
}

var a = new A;
assert.sameValue(a.g(), 14);
