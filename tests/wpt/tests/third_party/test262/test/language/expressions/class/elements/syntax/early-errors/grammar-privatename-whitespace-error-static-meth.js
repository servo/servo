// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-whitespace-error-static-meth.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: No space allowed between sigil and IdentifierName (Static Method) (class expression)
esid: prod-ClassElement
features: [class-static-methods-private, class]
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

var C = class {
  static # m() {}
};
