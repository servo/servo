// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-whitespace-error-static-field.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: No space allowed between sigil and IdentifierName (Static Field) (class declaration)
esid: prod-ClassElement
features: [class-static-fields-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Updated Productions

    ClassElementName :
      PropertyName
      PrivateName

    PrivateName ::
      # IdentifierName

---*/


$DONOTEVALUATE();

class C {
  static # x;
}
