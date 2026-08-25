// This file was procedurally generated from the following sources:
// - src/class-elements/eval-err-contains-supercall-2.case
// - src/class-elements/initializer-eval-super-call/cls-expr-private-fields-eval-nested.template
/*---
description: error if `super().x` in StatementList of eval (direct eval)
esid: sec-performeval-rules-in-initializer
features: [class, class-fields-public, class-fields-private]
flags: [generated]
info: |
    Additional Early Error Rules for Eval Outside Constructor Methods
    These static semantics are applied by PerformEval when a direct eval call occurs outside of the constructor method of a ClassDeclaration or ClassExpression.
    ScriptBody : StatementList

      It is a Syntax Error if StatementList Contains SuperCall.

---*/


var executed = false;
var A = class {}
var C = class extends A {
  #x = eval('executed = true; super().x;');
}

assert.throws(SyntaxError, function() {
  new C();
});

assert.sameValue(executed, false);
