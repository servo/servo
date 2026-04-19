// This file was procedurally generated from the following sources:
// - src/class-elements/eval-contains-superproperty-1.case
// - src/class-elements/initializer-eval-super-property/cls-decl-fields-indirect-eval-arrow-body.template
/*---
description: super.x in StatementList of eval (indirect eval)
esid: sec-performeval-rules-in-initializer
features: [class, class-fields-public]
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


class A {}
class C extends A {
  x = (0, eval)('() => super.x;');
}

assert.throws(SyntaxError, function() {
  new C().x();
});

