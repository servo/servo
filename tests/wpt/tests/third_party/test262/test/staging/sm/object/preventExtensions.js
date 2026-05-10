/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.preventExtensions() should return its argument with no conversion when the argument is a primitive value
info: bugzilla.mozilla.org/show_bug.cgi?id=1073446
esid: pending
features: [Symbol]
---*/

assert.sameValue(Object.preventExtensions(), undefined);
assert.sameValue(Object.preventExtensions(undefined), undefined);
assert.sameValue(Object.preventExtensions(null), null);
assert.sameValue(Object.preventExtensions(1), 1);
assert.sameValue(Object.preventExtensions("foo"), "foo");
assert.sameValue(Object.preventExtensions(true), true);
assert.sameValue(Object.preventExtensions(Symbol.for("foo")), Symbol.for("foo"));
