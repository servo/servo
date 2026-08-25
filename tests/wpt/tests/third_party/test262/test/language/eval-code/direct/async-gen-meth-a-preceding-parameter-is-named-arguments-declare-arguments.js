// This file was procedurally generated from the following sources:
// - src/direct-eval-code/declare-arguments.case
// - src/direct-eval-code/default/async-gen-meth/a-preceding-parameter-is-named-arguments.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when a preceding parameter is named |arguments|.)
esid: sec-evaldeclarationinstantiation
features: [globalThis]
flags: [generated, noStrict]
---*/

const oldArguments = globalThis.arguments;
let o = { async *f(arguments, p = eval("var arguments")) {
  
}};
assert.throws(SyntaxError, o.f);
assert.sameValue(globalThis.arguments, oldArguments, "globalThis.arguments unchanged");
