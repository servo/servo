/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.seal() should return its argument with no conversion when the argument is a primitive value
info: bugzilla.mozilla.org/show_bug.cgi?id=1075294
esid: pending
features: [Symbol]
---*/

assert.sameValue(Object.seal(), undefined);
assert.sameValue(Object.seal(undefined), undefined);
assert.sameValue(Object.seal(null), null);
assert.sameValue(Object.seal(1), 1);
assert.sameValue(Object.seal("foo"), "foo");
assert.sameValue(Object.seal(true), true);
assert.sameValue(Object.seal(Symbol.for("foo")), Symbol.for("foo"));
