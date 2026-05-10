// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-fields-same-line-error.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: SyntaxError (class expression)
esid: prod-ClassElement
features: [class-fields-public, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      MethodDefinition
      static MethodDefinition
      FieldDefinition ;
      ;

    FieldDefinition :
      ClassElementName Initializer _opt

    ClassElementName :
      PropertyName
      PrivateName

---*/


$DONOTEVALUATE();

var C = class {
  x y
};
