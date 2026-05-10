/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
assert.sameValue(Object.hasOwn({}, "any"), false);
assert.throws(TypeError, () => Object.hasOwn(null, "any"));

var x = { test: 'test value'}
var y = {}
var z = Object.create(x);

assert.sameValue(Object.hasOwn(x, "test"), true);
assert.sameValue(Object.hasOwn(y, "test"), false);
assert.sameValue(Object.hasOwn(z, "test"), false);

