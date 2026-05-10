/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
(function () {
    var [x, y] = [1, function () { return x; }];
    assert.sameValue(y(), 1);
})();

