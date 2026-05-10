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
 * In strict mode, the identifier bound by a 'catch' clause may not
 * be 'eval' or 'arguments'.
 */
assert.sameValue(testLenientAndStrict('try{}catch(eval){}',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('try{}catch([eval]){}',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('try{}catch({x:eval}){}',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('try{}catch(arguments){}',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('try{}catch([arguments]){}',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('try{}catch({x:arguments}){}',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

