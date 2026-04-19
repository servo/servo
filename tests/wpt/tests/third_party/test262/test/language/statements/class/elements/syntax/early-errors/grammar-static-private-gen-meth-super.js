// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-static-private-gen-meth-super.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: Static Generator Private Methods cannot contain direct super (class declaration)
esid: prod-ClassElement
features: [generators, class-static-methods-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Class Definitions / Static Semantics: Early Errors

    ClassElement : static MethodDefinition
        It is a Syntax Error if HasDirectSuper of MethodDefinition is true.

---*/


$DONOTEVALUATE();

class C extends Function{
  static * #method() {
      super();
  }
}
