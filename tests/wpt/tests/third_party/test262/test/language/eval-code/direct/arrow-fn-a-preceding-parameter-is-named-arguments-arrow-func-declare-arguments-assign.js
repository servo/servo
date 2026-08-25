// This file was procedurally generated from the following sources:
// - src/direct-eval-code/arrow-func-declare-arguments-assign.case
// - src/direct-eval-code/arrow-func/a-preceding-parameter-is-named-arguments.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when a preceding parameter is named |arguments|.)
esid: sec-evaldeclarationinstantiation
flags: [generated, noStrict]
---*/

const oldArguments = globalThis.arguments;
const f = (arguments, p = eval("var arguments = 'param'")) => {}
assert.throws(SyntaxError, f);
assert.sameValue(globalThis.arguments, oldArguments, "globalThis.arguments unchanged");
