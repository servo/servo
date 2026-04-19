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
 * 'with' statements are forbidden in strict top-level code. This is
 * eval code, but that's close enough.
 */
assert.sameValue(testLenientAndStrict('with (1) {}',
                              completesNormally,
                              raisesException(SyntaxError)),
         true);

/* 'with' statements are forbidden in strict function code. */
assert.sameValue(testLenientAndStrict('function f() { "use strict"; with (1) {} }',
                              parseRaisesException(SyntaxError),
                              parseRaisesException(SyntaxError)),
         true);
                              
/*
 * A use strict directive in a function mustn't affect the strictness
 * of subsequent code.
 */
assert.sameValue(parsesSuccessfully('function f() { "use strict"; }; with (1) {}'),
         true);

