// This file was procedurally generated from the following sources:
// - src/direct-eval-code/declare-arguments.case
// - src/direct-eval-code/default/gen-func-expr-named/fn-body-cntns-arguments-lex-bind.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when the function body contains an |arguments| lexical binding.)
esid: sec-evaldeclarationinstantiation
flags: [generated, noStrict]
---*/



assert.sameValue("arguments" in this, false, "No global 'arguments' binding");

let f = function * f(p = eval("var arguments")) {
  let arguments;
}
assert.throws(SyntaxError, f);

assert.sameValue("arguments" in this, false, "No global 'arguments' binding");
