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
assert.sameValue(testLenientAndStrict('var r = /foo/; r.lastIndex = 42; r.lastIndex',
                              returns(42), returns(42)),
         true);
assert.sameValue(testLenientAndStrict('var r = /foo/; delete r.lastIndex',
                              returns(false), raisesException(TypeError)),
         true);

