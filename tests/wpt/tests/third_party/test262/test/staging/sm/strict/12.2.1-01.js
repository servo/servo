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
 * In strict mode code, 'let' and 'const' declarations may not bind
 * 'eval' or 'arguments'.
 */
assert.sameValue(testLenientAndStrict('let eval;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('let x,eval;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('let arguments;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('let x,arguments;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('const eval = undefined;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('const x = undefined,eval = undefined;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('const arguments = undefined;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('const x = undefined,arguments = undefined;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

/*
 * In strict mode code, 'let' declarations appearing in 'for'
 * or 'for in' statements may not bind 'eval' or 'arguments'.
 */
assert.sameValue(testLenientAndStrict('for (let eval in [])break;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('for (let [eval] in [])break;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('for (let {x:eval} in [])break;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('for (let arguments in [])break;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('for (let [arguments] in [])break;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict('for (let {x:arguments} in [])break;',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

