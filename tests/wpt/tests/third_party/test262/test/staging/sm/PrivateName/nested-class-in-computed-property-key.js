// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
let capturedPrivateAccess;
class A {
  // Declare private name in outer class.
  static #x = 42;

  static [(
    // Inner class in computed property key.
    class {},

    // Access to private name from outer class.
    capturedPrivateAccess = () => A.#x
  )];
}
assert.sameValue(capturedPrivateAccess(), 42);

