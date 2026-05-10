/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.isSealed() should return true when given primitive values as input
info: bugzilla.mozilla.org/show_bug.cgi?id=1062860
esid: pending
features: [Symbol]
---*/

assert.sameValue(Object.isSealed(), true);
assert.sameValue(Object.isSealed(undefined), true);
assert.sameValue(Object.isSealed(null), true);
assert.sameValue(Object.isSealed(1), true);
assert.sameValue(Object.isSealed("foo"), true);
assert.sameValue(Object.isSealed(true), true);
assert.sameValue(Object.isSealed(Symbol()), true);
