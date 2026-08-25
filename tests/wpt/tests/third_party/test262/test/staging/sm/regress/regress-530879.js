/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
function* f(a, b, c, d) {
    yield arguments.length;
}
assert.sameValue(0, f().next().value, "bug 530879");
