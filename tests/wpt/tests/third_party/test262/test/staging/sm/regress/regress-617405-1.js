/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
function C(){}
C.prototype = 1;
assert.sameValue(Object.getOwnPropertyDescriptor(C, "prototype").configurable, false);

