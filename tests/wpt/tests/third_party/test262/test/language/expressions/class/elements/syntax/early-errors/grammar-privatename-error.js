// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-error.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: No space allowed between sigial and IdentifierName (class expression)
esid: prod-ClassElement
features: [class-fields-private, class]
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

    PrivateName ::
      # IdentifierName

---*/


$DONOTEVALUATE();

var C = class {
  # x
};
