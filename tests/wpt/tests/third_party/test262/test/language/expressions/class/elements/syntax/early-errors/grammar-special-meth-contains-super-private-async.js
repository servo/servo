// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-special-meth-contains-super-private-async.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: Private Async Methods cannot contain direct super (class expression)
esid: prod-ClassElement
features: [async-functions, class-methods-private, class]
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

var C = class extends Function{
  async #method() {
      super();
  }
};
