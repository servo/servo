// This file was procedurally generated from the following sources:
// - src/direct-eval-code/declare-arguments.case
// - src/direct-eval-code/default/async-gen-func-expr/fn-body-cntns-arguments-func-decl.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when the function body contains an |arguments| function declaration.)
esid: sec-evaldeclarationinstantiation
features: [globalThis]
flags: [generated, noStrict]
---*/

const oldArguments = globalThis.arguments;
let f = async function * (p = eval("var arguments")) {
  function arguments() {}
}

assert.throws(SyntaxError, f);
assert.sameValue(globalThis.arguments, oldArguments, "globalThis.arguments unchanged");
