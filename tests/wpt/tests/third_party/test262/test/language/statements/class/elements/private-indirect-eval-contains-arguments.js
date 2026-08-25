// This file was procedurally generated from the following sources:
// - src/class-elements/eval-contains-arguments.case
// - src/class-elements/initializer-indirect-eval-arguments/cls-decl-private-fields-indirect-eval.template
/*---
description: No error if `arguments` in StatementList of eval (indirect eval)
esid: sec-performeval-rules-in-initializer
features: [class, class-fields-public, class-fields-private]
flags: [generated, noStrict]
info: |
    For indirect eval, the "Additional Early Error Rules for Eval Inside Initializer"
    (in #sec-performeval-rules-in-initializer) are NOT applicable.

---*/


var arguments = 1;
class C {
  #x = (0, eval)('arguments;');
  x() {
    return this.#x;
  }
}
assert.sameValue(new C().x(), arguments);
