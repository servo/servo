// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-static-private-async-gen-meth-constructor.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: Static Async Generator Private Methods cannot be named constructor (class expression)
esid: prod-ClassElement
features: [async-iteration, class-static-methods-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Class Definitions / Static Semantics: Early Errors

    ClassElementName : PrivateName ;
        It is a Syntax Error if StringValue of PrivateName is "#constructor"

---*/


$DONOTEVALUATE();

var C = class {
  static async * #constructor() {}
};
