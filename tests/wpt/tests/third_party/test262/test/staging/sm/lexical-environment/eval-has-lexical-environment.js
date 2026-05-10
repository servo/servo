/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Eval always has a lexical environment
info: bugzilla.mozilla.org/show_bug.cgi?id=1193583
esid: pending
---*/

eval(`
let foo = 42;
const kay = foo;
var bar = 84;
function f() {
  return foo + kay;
}
     `);

(1, eval)(`
let foo2 = 42;
const kay2 = foo2;
`);

// Lexical declarations should not have escaped eval.
assert.sameValue(typeof foo, "undefined");
assert.sameValue(typeof kay, "undefined");
assert.sameValue(typeof foo2, "undefined");
assert.sameValue(typeof kay2, "undefined");

// Eval'd functions can close over lexical bindings.
assert.sameValue(f(), 84);

// Var can escape direct eval.
assert.sameValue(bar, 84);
