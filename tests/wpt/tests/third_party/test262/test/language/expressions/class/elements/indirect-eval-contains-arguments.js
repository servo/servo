// This file was procedurally generated from the following sources:
// - src/class-elements/eval-contains-arguments.case
// - src/class-elements/initializer-indirect-eval-arguments/cls-expr-fields-indirect-eval.template
/*---
description: No error if `arguments` in StatementList of eval (indirect eval)
esid: sec-performeval-rules-in-initializer
features: [class, class-fields-public]
flags: [generated, noStrict]
info: |
    For indirect eval, the "Additional Early Error Rules for Eval Inside Initializer"
    (in #sec-performeval-rules-in-initializer) are NOT applicable.

---*/


var arguments = 1;
var executed = false;
var C = class {
  x = (0, eval)('arguments;');
}

assert.sameValue(new C().x, arguments);
