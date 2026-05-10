/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.isFrozen() should return true when given primitive values as input
info: bugzilla.mozilla.org/show_bug.cgi?id=1071464
esid: pending
features: [Symbol]
---*/

assert.sameValue(Object.isFrozen(), true);
assert.sameValue(Object.isFrozen(undefined), true);
assert.sameValue(Object.isFrozen(null), true);
assert.sameValue(Object.isFrozen(1), true);
assert.sameValue(Object.isFrozen("foo"), true);
assert.sameValue(Object.isFrozen(true), true);
assert.sameValue(Object.isFrozen(Symbol()), true);
