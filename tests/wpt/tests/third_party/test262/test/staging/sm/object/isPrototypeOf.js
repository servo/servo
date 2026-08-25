/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.prototype.isPrototypeOf
info: bugzilla.mozilla.org/show_bug.cgi?id=619283
esid: pending
---*/

var isPrototypeOf = Object.prototype.isPrototypeOf;

/*
 * 1. If V is not an Object, return false.
 */
assert.sameValue(isPrototypeOf(), false);
assert.sameValue(isPrototypeOf(1), false);
assert.sameValue(isPrototypeOf(Number.MAX_VALUE), false);
assert.sameValue(isPrototypeOf(NaN), false);
assert.sameValue(isPrototypeOf(""), false);
assert.sameValue(isPrototypeOf("sesquicentennial"), false);
assert.sameValue(isPrototypeOf(true), false);
assert.sameValue(isPrototypeOf(false), false);
assert.sameValue(isPrototypeOf(0.72), false);
assert.sameValue(isPrototypeOf(undefined), false);
assert.sameValue(isPrototypeOf(null), false);


/*
 * 2. Let O be the result of calling ToObject passing the this value as the
 *    argument.
 */
var protoGlobal = Object.create(this);
assert.throws(TypeError, function() { isPrototypeOf.call(null, {}); });
assert.throws(TypeError, function() { isPrototypeOf.call(undefined, {}); });
assert.throws(TypeError, function() { isPrototypeOf({}); });
assert.throws(TypeError, function() { isPrototypeOf.call(null, protoGlobal); });
assert.throws(TypeError, function() { isPrototypeOf.call(undefined, protoGlobal); });
assert.throws(TypeError, function() { isPrototypeOf(protoGlobal); });


/*
 * 3. Repeat
 */

/*
 * 3a. Let V be the value of the [[Prototype]] internal property of V.
 * 3b. If V is null, return false.
 */
assert.sameValue(Object.prototype.isPrototypeOf(Object.prototype), false);
assert.sameValue(String.prototype.isPrototypeOf({}), false);
assert.sameValue(Object.prototype.isPrototypeOf(Object.create(null)), false);

/* 3c. If O and V refer to the same object, return true. */
assert.sameValue(Object.prototype.isPrototypeOf({}), true);
assert.sameValue(this.isPrototypeOf(protoGlobal), true);
assert.sameValue(Object.prototype.isPrototypeOf({}), true);
assert.sameValue(Object.prototype.isPrototypeOf(new Number(17)), true);
assert.sameValue(Object.prototype.isPrototypeOf(function(){}), true);
