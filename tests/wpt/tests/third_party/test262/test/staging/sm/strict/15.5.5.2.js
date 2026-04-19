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
assert.sameValue(testLenientAndStrict('"foo"[0] = 1',
                              returns(1), raisesException(TypeError)),
         true);
assert.sameValue(testLenientAndStrict('delete "foo"[0]',
                              returns(false), raisesException(TypeError)),
         true);

