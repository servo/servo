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
// Ordinary function definitions should be unaffected.
assert.sameValue(testLenientAndStrict("function f() { }",
                              parsesSuccessfully,
                              parsesSuccessfully),
         true);

// Lambdas are always permitted within blocks.
assert.sameValue(testLenientAndStrict("{ (function f() { }) }",
                              parsesSuccessfully,
                              parsesSuccessfully),
         true);

// Function statements within unbraced blocks are forbidden in strict mode code.
// They are allowed only under if statements in sloppy mode.
assert.sameValue(testLenientAndStrict("if (true) function f() { }",
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict("while (true) function f() { }",
                              parseRaisesException(SyntaxError),
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict("do function f() { } while (true);",
                              parseRaisesException(SyntaxError),
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict("for(;;) function f() { }",
                              parseRaisesException(SyntaxError),
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict("for(x in []) function f() { }",
                              parseRaisesException(SyntaxError),
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict("with(o) function f() { }",
                              parseRaisesException(SyntaxError),
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict("switch(1) { case 1: function f() { } }",
                              parsesSuccessfully,
                              parsesSuccessfully),
         true);
assert.sameValue(testLenientAndStrict("x: function f() { }",
                              parsesSuccessfully,
                              parseRaisesException(SyntaxError)),
         true);
assert.sameValue(testLenientAndStrict("try { function f() { } } catch (x) { }",
                              parsesSuccessfully,
                              parsesSuccessfully),
         true);

// Lambdas are always permitted within any sort of statement.
assert.sameValue(testLenientAndStrict("if (true) (function f() { })",
                              parsesSuccessfully,
                              parsesSuccessfully),
         true);

// Function statements are permitted in blocks within lenient functions.
assert.sameValue(parsesSuccessfully("function f() { function g() { } }"),
         true);

// Function statements are permitted in if statement within lenient functions.
assert.sameValue(parsesSuccessfully("function f() { if (true) function g() { } }"),
         true);

assert.sameValue(parseRaisesException(SyntaxError)
         ("function f() { 'use strict'; if (true) function g() { } }"),
         true);

assert.sameValue(parsesSuccessfully("function f() { 'use strict'; { function g() { } } }"),
         true);

assert.sameValue(parsesSuccessfully("function f() { 'use strict'; if (true) (function g() { }) }"),
         true);

assert.sameValue(parsesSuccessfully("function f() { 'use strict'; { (function g() { }) } }"),
         true);

// Eval should behave the same way. (The parse-only tests use the Function constructor.)
assert.sameValue(testLenientAndStrict("function f() { }",
                              completesNormally,
                              completesNormally),
         true);
assert.sameValue(testLenientAndStrict("{ function f() { } }",
                              completesNormally,
                              completesNormally),
         true);

