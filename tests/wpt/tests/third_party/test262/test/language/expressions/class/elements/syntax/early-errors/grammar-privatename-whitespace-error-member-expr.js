// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-whitespace-error-member-expr.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: No space allowed between sigil and IdentifierName (MemberExpression) (class expression)
esid: prod-ClassElement
features: [class-fields-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Updated Productions

    MemberExpression :
      MemberExpression . PrivateName

    CallExpression :
      CallExpression . PrivateName

    PrivateName ::
      # IdentifierName

---*/


$DONOTEVALUATE();

var C = class {
  #x;
  m() {
    this.# x;
  }
};
