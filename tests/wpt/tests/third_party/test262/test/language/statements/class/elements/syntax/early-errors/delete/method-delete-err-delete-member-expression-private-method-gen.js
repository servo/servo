// This file was procedurally generated from the following sources:
// - src/class-elements/err-delete-member-expression-private-method-gen.case
// - src/class-elements/delete-error/cls-decl-method-delete.template
/*---
description: It's a SyntaxError if delete operator is applied to MemberExpression.PrivateName generator (in method)
esid: sec-class-definitions-static-semantics-early-errors
features: [class-methods-private, generators, class, class-fields-private]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    This file must never use the onlyStrict flag as the strict mode should always
    be observed inside class bodies.

    Static Semantics: Early Errors

      UnaryExpression : delete UnaryExpression

      It is a Syntax Error if the UnaryExpression is contained in strict mode
      code and the derived UnaryExpression is
      PrimaryExpression : IdentifierReference ,
      MemberExpression : MemberExpression.PrivateName , or
      CallExpression : CallExpression.PrivateName .

---*/


$DONOTEVALUATE();

class C {
  #x;

  x() {
    
    delete this.#m
;
  }

  *#m() {}
}
