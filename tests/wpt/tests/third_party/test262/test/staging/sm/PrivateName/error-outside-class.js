// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Can't reference a private field without an object
assert.throws(SyntaxError, () => eval('#x'));

// Can't reference a private field without an enclosing class
assert.throws(SyntaxError, () => eval('this.#x'));

// Can't reference a private field in a random function outside a class context
assert.throws(SyntaxError, () => eval('function foo() { return this.#x'));


