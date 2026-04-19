// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-static-private-async-meth-constructor.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: Static Async Private Methods cannot be named constructor (class declaration)
esid: prod-ClassElement
features: [async-functions, class-static-methods-private, class]
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

class C {
  static async #constructor() {}
}
