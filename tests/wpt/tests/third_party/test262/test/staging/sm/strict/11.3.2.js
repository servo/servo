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
 * Postfix decrement expressions must not have 'eval' or 'arguments'
 * as their operands.
 */
assert.sameValue(testLenientAndStrict('arguments--',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('eval--',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('(arguments)--',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('(eval)--',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

