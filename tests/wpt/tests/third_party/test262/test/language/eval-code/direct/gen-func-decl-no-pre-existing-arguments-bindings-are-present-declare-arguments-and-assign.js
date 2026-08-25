// This file was procedurally generated from the following sources:
// - src/direct-eval-code/declare-arguments-and-assign.case
// - src/direct-eval-code/default/gen-func-decl/no-pre-existing-arguments-bindings-are-present.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when no pre-existing |arguments| bindings are present.)
esid: sec-evaldeclarationinstantiation
flags: [generated, noStrict]
---*/


assert.sameValue("arguments" in this, false, "No global 'arguments' binding");

function * f(p = eval("var arguments = 'param'")) {}
assert.throws(SyntaxError, f);

assert.sameValue("arguments" in this, false, "No global 'arguments' binding");
