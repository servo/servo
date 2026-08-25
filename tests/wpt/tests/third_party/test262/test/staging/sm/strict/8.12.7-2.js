/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/non262-strict-shell.js]
description: |
  pending
esid: pending
---*/
// delete o[p] only performs ToString(p) once, even if there's a strict error.
var hits = 0;
var p = {
    toString: function () {
        hits++;
        return "noconfig";
    }
};
assert.sameValue(testLenientAndStrict('var o = Object.freeze({noconfig: "ow"}); delete o[p]',
                              returns(false), raisesException(TypeError)),
         true);
assert.sameValue(hits, 2);

