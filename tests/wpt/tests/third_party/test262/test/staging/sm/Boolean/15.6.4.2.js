/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

assert.throws(TypeError, function() { Boolean.prototype.toString.call(42); });
assert.throws(TypeError, function() { Boolean.prototype.toString.call(""); });
assert.throws(TypeError, function() { Boolean.prototype.toString.call({}); });
assert.throws(TypeError, function() { Boolean.prototype.toString.call(null); });
assert.throws(TypeError, function() { Boolean.prototype.toString.call([]); });
assert.throws(TypeError, function() { Boolean.prototype.toString.call(undefined); });
assert.throws(TypeError, function() { Boolean.prototype.toString.call(new String()); });

assert.sameValue(Boolean.prototype.toString.call(true), "true");
assert.sameValue(Boolean.prototype.toString.call(new Boolean(true)), "true");
