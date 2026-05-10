// This file was procedurally generated from the following sources:
// - src/direct-eval-code/declare-arguments-and-assign.case
// - src/direct-eval-code/default/async-func-decl/a-preceding-parameter-is-named-arguments.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when a preceding parameter is named |arguments|.)
esid: sec-evaldeclarationinstantiation
features: [globalThis]
flags: [generated, async, noStrict]
---*/

const oldArguments = globalThis.arguments;
async function f(arguments, p = eval("var arguments = 'param'")) {
  
}
f().then($DONE, error => {
  assert(error instanceof SyntaxError);
  assert.sameValue(globalThis.arguments, oldArguments, "globalThis.arguments unchanged");
}).then($DONE, $DONE);

