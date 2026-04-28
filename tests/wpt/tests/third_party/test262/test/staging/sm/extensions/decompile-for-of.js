// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// The decompiler can handle the implicit call to @@iterator in a for-of loop.

var x = {};
assert.throws(TypeError, () => eval("for (var v of x) throw fit;"), "x is not iterable");
assert.throws(TypeError, () => eval("[...x]"), "x is not iterable");
assert.throws(TypeError, () => eval("Math.hypot(...x)"), "x is not iterable");

x[Symbol.iterator] = "potato";
assert.throws(TypeError, () => eval("for (var v of x) throw fit;"), "x is not iterable");

x[Symbol.iterator] = {};
assert.throws(TypeError, () => eval("for (var v of x) throw fit;"), "x[Symbol.iterator] is not a function");

