// This file was procedurally generated from the following sources:
// - src/direct-eval-code/arrow-func-declare-arguments-assign.case
// - src/direct-eval-code/arrow-func/a-following-parameter-is-named-arguments.template
/*---
description: Declare "arguments" and assign to it in direct eval code (Declare |arguments| when a following parameter is named |arguments|.)
esid: sec-evaldeclarationinstantiation
flags: [generated, noStrict]
---*/

const oldArguments = globalThis.arguments;
const f = (p = eval("var arguments = 'param'"), arguments) => {}
assert.throws(SyntaxError, f);
assert.sameValue(globalThis.arguments, oldArguments, "globalThis.arguments unchanged");
