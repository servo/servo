/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Number.EPSILON
info: bugzilla.mozilla.org/show_bug.cgi?id=885798
esid: pending
---*/

// Test value
assert.sameValue(Number.EPSILON, Math.pow(2, -52));

// Test property attributes
var descriptor = Object.getOwnPropertyDescriptor(Number, 'EPSILON');
assert.sameValue(descriptor.writable, false);
assert.sameValue(descriptor.configurable, false);
assert.sameValue(descriptor.enumerable, false);
