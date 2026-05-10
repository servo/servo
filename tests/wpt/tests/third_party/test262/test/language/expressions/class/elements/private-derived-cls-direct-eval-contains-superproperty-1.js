// This file was procedurally generated from the following sources:
// - src/class-elements/eval-contains-superproperty-1.case
// - src/class-elements/initializer-eval-super-property/cls-expr-private-fields-eval.template
/*---
description: super.x in StatementList of eval (direct eval)
esid: sec-performeval-rules-in-initializer
features: [class, class-fields-public, class-fields-private]
flags: [generated]
info: |
    Additional Early Error Rules for Eval Inside Initializer
    These static semantics are applied by PerformEval when a direct eval call occurs inside a class field initializer.
    ScriptBody : StatementList

      ...
      The remaining eval rules apply as outside a constructor, inside a method, and inside a function.

    Additional Early Error Rules for Eval Outside Methods
    These static semantics are applied by PerformEval when a direct eval call occurs outside of a MethodDefinition.
    ScriptBody : StatementList

      It is a Syntax Error if StatementList Contains SuperProperty.

---*/


var executed = false;
var A = class {}
var C = class extends A {
  #x = eval('executed = true; super.x;');
};

new C();

assert.sameValue(executed, true);
