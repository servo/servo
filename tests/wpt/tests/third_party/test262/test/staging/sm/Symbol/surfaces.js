/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// Test superficial properties of the Symbol constructor and prototype.

var desc = Object.getOwnPropertyDescriptor(this, "Symbol");
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.writable, true);
assert.sameValue(typeof Symbol, "function");
assert.sameValue(Symbol.length, 0);

desc = Object.getOwnPropertyDescriptor(Symbol, "prototype");
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.writable, false);

assert.sameValue(Symbol.prototype.constructor, Symbol);
desc = Object.getOwnPropertyDescriptor(Symbol.prototype, "constructor");
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.writable, true);

desc = Object.getOwnPropertyDescriptor(Symbol, "iterator");
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.writable, false);

assert.sameValue(Symbol.for.length, 1);
assert.sameValue(Symbol.prototype.toString.length, 0);
assert.sameValue(Symbol.prototype.valueOf.length, 0);

