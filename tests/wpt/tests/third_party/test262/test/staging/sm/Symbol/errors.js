/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// Section numbers cite ES6 rev 24 (2014 April 27).

var sym = Symbol();

// 7.2.2 IsCallable
assert.throws(TypeError, () => sym());
assert.throws(TypeError, () => Function.prototype.call.call(sym));

// 7.2.5 IsConstructor
assert.throws(TypeError, () => new sym());
assert.throws(TypeError, () => new Symbol());

