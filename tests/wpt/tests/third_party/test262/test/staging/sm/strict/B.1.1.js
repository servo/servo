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
/* Octal integer literal at top level. */
assert.sameValue(testLenientAndStrict('010',
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);

/* Octal integer literal in strict function body */
assert.sameValue(parseRaisesException(SyntaxError)
         ('function f() { "use strict"; 010; }'),
         true);
                              

/*
 * Octal integer literal after strict function body (restoration of
 * scanner state)
 */
assert.sameValue(parsesSuccessfully('function f() { "use strict"; }; 010'),
         true);

/* Octal integer literal in function body */
assert.sameValue(parsesSuccessfully('function f() { 010; }'),
         true);

