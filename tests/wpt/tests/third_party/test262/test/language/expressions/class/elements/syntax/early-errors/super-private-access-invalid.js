// This file was procedurally generated from the following sources:
// - src/class-elements/super-private-access-invalid.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: It is syntax error if PrivateName IdentifierName is accessed on SuperProperty (class expression)
esid: prod-ClassElement
features: [class-methods-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElementName :
      PropertyName
      PrivateName

    PrivateName ::
      # IdentifierName

    SuperProperty:
      super[Expression]
      super.IdentifierName

---*/


$DONOTEVALUATE();

var C = class extends B
{
  #x() {}

  method() {
    super.#x();
  }
};
