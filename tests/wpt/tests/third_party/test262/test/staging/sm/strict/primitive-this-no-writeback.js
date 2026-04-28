/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
/* Verify that GETTHISPROP does not update the frame's |this| slot. */

var f = String.prototype.m = function () {
    "use strict";
    assert.sameValue(this, "s");
    // The GETTHISPROP should not cause |this| to become wrapped.
    return [this.m, this];
};
var a = "s".m();
assert.sameValue(a[0], f);
assert.sameValue(a[1], "s");

