// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-special-meth-contains-super-async.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: Async Methods cannot contain direct super (class declaration)
esid: prod-ClassElement
features: [async-functions, class]
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
  async method() {
      super();
  }
}
