/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
function test(arg) {
    eval(arg);
    {
        function arguments() { return 1; }
    }
    return arguments;
}

assert.sameValue("function", typeof test("42"), "function sub-statement must override arguments");
