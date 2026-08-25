/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

assert.throws(TypeError, function() { String.prototype.toString.call(42); });
assert.throws(TypeError, function() { String.prototype.toString.call(true); });
assert.throws(TypeError, function() { String.prototype.toString.call({}); });
assert.throws(TypeError, function() { String.prototype.toString.call(null); });
assert.throws(TypeError, function() { String.prototype.toString.call([]); });
assert.throws(TypeError, function() { String.prototype.toString.call(undefined); });
assert.sameValue(String.prototype.toString.call(""), "");
