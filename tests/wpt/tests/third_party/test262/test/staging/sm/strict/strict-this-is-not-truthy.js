/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

// See bug 630543.

function f() {
    "use strict";
    return !this;
}
assert.sameValue(f.call(null), true);

