/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  Coerce the argument passed to Object.keys using ToObject
info: bugzilla.mozilla.org/show_bug.cgi?id=1038545
esid: pending
features: [Symbol]
---*/

assert.throws(TypeError, () => Object.keys());
assert.throws(TypeError, () => Object.keys(undefined));
assert.throws(TypeError, () => Object.keys(null));

assert.compareArray(Object.keys(1), []);
assert.compareArray(Object.keys(true), []);
assert.compareArray(Object.keys(Symbol("foo")), []);

assert.compareArray(Object.keys("foo"), ["0", "1", "2"]);
