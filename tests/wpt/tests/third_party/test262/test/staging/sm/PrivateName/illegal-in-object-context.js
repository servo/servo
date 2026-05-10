// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Private names aren't valid in object literals.

assert.throws(SyntaxError, () => eval(`var o = {#a: 0};`));
assert.throws(SyntaxError, () => eval(`var o = {#a};`));
assert.throws(SyntaxError, () => eval(`var o = {#a(){}};`));
assert.throws(SyntaxError, () => eval(`var o = {get #a(){}};`));
assert.throws(SyntaxError, () => eval(`var o = {set #a(v){}};`));
assert.throws(SyntaxError, () => eval(`var o = {*#a(v){}};`));
assert.throws(SyntaxError, () => eval(`var o = {async #a(v){}};`));
assert.throws(SyntaxError, () => eval(`var o = {async *#a(v){}};`));

