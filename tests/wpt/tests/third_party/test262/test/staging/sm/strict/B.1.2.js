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
assert.sameValue(testLenientAndStrict('"\\010"',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

assert.sameValue(testLenientAndStrict('"\\00"',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

assert.sameValue(testLenientAndStrict('"\\1"',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

assert.sameValue(testLenientAndStrict('"\\08"',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

assert.sameValue(testLenientAndStrict('"\\0"',
                              parsesSuccessfully,
                              parsesSuccessfully),
         true);

assert.sameValue(testLenientAndStrict('"\\0x"',
                              parsesSuccessfully,
                              parsesSuccessfully),
         true);

