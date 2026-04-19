/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

for (var thisv of [null, undefined, false, true, 0, ""]) {
  assert.throws(TypeError, () => Map.prototype.values.call(thisv));
  assert.throws(TypeError, () => Map.prototype.keys.call(thisv));
  assert.throws(TypeError, () => Map.prototype.entries.call(thisv));
  assert.throws(TypeError, () => Map.prototype[Symbol.iterator].call(thisv));
}

