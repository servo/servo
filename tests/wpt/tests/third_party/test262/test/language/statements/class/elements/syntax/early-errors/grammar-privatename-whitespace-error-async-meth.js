// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-whitespace-error-async-meth.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: No space allowed between sigil and IdentifierName (Async Method) (class declaration)
esid: prod-ClassElement
features: [async-functions, class-methods-private, class]
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
  async # m() {}
}
