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
assert.sameValue(testLenientAndStrict('function f(eval,[x]){}',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

