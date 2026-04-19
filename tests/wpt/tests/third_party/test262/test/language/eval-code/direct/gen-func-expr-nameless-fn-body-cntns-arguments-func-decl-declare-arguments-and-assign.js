// This file was procedurally generated from the following sources:
// - src/direct-eval-code/declare-arguments-and-assign.case
// - src/direct-eval-code/default/gen-func-expr-nameless/fn-body-cntns-arguments-func-decl.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when the function body contains an |arguments| function declaration.)
esid: sec-evaldeclarationinstantiation
flags: [generated, noStrict]
---*/


assert.sameValue("arguments" in this, false, "No global 'arguments' binding");

let f = function * (p = eval("var arguments = 'param'")) {
  function arguments() {}
}
assert.throws(SyntaxError, f);


assert.sameValue("arguments" in this, false, "No global 'arguments' binding");
