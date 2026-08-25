/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var BadSyntaxStrings = [
    "function foo1() { \"use strict\"; try {} catch (eval) {} }",
    "function foo2() { \"use strict\"; let eval = 9; foo(); }",
    "function foo3() { \"use strict\"; for (let eval = 3;;) { foo(); }}",
    "function foo4() { \"use strict\"; for (let eval in {a:1}) { foo(); }}",
    "function foo5() { \"use strict\"; for (let eval of [1, 2, 3]) { foo(); }}",
    "function foo6() { \"use strict\"; var eval = 12; }",
    "function foo7() { \"use strict\"; for (var eval = 3;;) { foo(); }}",
    "function foo8() { \"use strict\"; for (var eval in {a:1}) { foo(); }}",
    "function foo9() { \"use strict\"; for (var eval of [1, 2, 3]) { foo(); }}",
    "function foo10() { \"use strict\"; const eval = 12; }",
    "function foo11() { \"use strict\"; for (const eval = 3;;) { foo(); }}",
    "function foo12() { \"use strict\"; return [eval for (eval of [1, 2, 3])]; }",
    "function foo13() { \"use strict\"; return [eval for (eval in {a:3})]; }",
    "function foo14() { \"use strict\"; return (eval for (eval of [1, 2, 3])); }",
    "function foo15() { \"use strict\"; return (eval for (eval in {a:3})); }"
];

function testString(s, i) {
    assert.throws(SyntaxError, function() {
        eval(s);
    });
}

for (var i = 0; i < BadSyntaxStrings.length; i++)
    testString(BadSyntaxStrings[i], i);
