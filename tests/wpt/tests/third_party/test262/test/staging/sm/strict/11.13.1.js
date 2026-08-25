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
/*
 * Simple assignment expressions in strict mode code must not be
 * assignments to 'eval' or 'arguments'.
 */
assert.sameValue(testLenientAndStrict('arguments=1',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('eval=1',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('(arguments)=1',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('(eval)=1',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

