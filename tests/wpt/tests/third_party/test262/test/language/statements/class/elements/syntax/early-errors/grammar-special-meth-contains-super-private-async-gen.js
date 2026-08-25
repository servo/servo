// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-special-meth-contains-super-private-async-gen.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: Private Async Generators Methods cannot contain direct super (class declaration)
esid: prod-ClassElement
features: [async-iteration, class-methods-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Class Definitions / Static Semantics: Early Errors

    ClassElement : MethodDefinition
        It is a Syntax Error if PropName of MethodDefinition is not "constructor" and HasDirectSuper of MethodDefinition is true.

---*/


$DONOTEVALUATE();

class C extends Function{
  async * #method() {
      super();
  }
}
