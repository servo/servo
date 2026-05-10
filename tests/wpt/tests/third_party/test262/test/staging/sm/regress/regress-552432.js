/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
(function (y) {
    arguments.y = 2;
    var x = Object.create(arguments);
    x[0] = 3;
    assert.sameValue(x[0], 3);
    assert.sameValue(x.y, 2);
    assert.sameValue(y, 1);
})(1);

