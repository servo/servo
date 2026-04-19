// This file was procedurally generated from the following sources:
// - src/class-elements/private-member-exp-cannot-escape-token.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: The pound signal in the private reference cannot be escaped (class expression)
esid: prod-ClassElement
features: [class-fields-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    PrivateName ::
      # IdentifierName

    MemberExpression :
      MemberExpression . PrivateName

    CallExpression :
      CallExpression . PrivateName

    U+0023 is the escape sequence for #

---*/


$DONOTEVALUATE();

var C = class {
  method() {
    this.\u0023field;
  }
};
