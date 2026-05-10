// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-constructor.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: Private Fields cannot be named constructor (class expression)
esid: prod-ClassElement
features: [class-fields-private, class]
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
  #constructor
};
