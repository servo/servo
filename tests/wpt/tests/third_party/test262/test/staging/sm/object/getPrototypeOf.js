/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Coerce the argument passed to Object.getPrototypeOf using ToObject
info: bugzilla.mozilla.org/show_bug.cgi?id=1079090
esid: pending
features: [Symbol]
---*/

assert.throws(TypeError, () => Object.getPrototypeOf());
assert.throws(TypeError, () => Object.getPrototypeOf(undefined));
assert.throws(TypeError, () => Object.getPrototypeOf(null));

assert.sameValue(Object.getPrototypeOf(1), Number.prototype);
assert.sameValue(Object.getPrototypeOf(true), Boolean.prototype);
assert.sameValue(Object.getPrototypeOf("foo"), String.prototype);
assert.sameValue(Object.getPrototypeOf(Symbol("foo")), Symbol.prototype);
