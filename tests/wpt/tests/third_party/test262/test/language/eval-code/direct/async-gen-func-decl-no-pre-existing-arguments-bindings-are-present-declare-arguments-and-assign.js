// This file was procedurally generated from the following sources:
// - src/direct-eval-code/declare-arguments-and-assign.case
// - src/direct-eval-code/default/async-gen-func-decl/no-pre-existing-arguments-bindings-are-present.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when no pre-existing |arguments| bindings are present.)
esid: sec-evaldeclarationinstantiation
features: [globalThis]
flags: [generated, noStrict]
---*/

const oldArguments = globalThis.arguments;
async function * f(p = eval("var arguments = 'param'")) {}

assert.throws(SyntaxError, f);
assert.sameValue(globalThis.arguments, oldArguments, "globalThis.arguments unchanged");
