// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-in-computed-property-missing.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: Use of undeclared PrivateName in ComputedProperty is a syntax error (class expression)
esid: prod-ClassElement
features: [class-fields-private, class-fields-public, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElementName:
      PropertyName
      PrivateIdentifier

    PropertyName:
      LiteralPropertyName
      ComputedPropertyName

    ComputedPropertyName:
      [ AssignmentExpression ]

    AssignmentExpression ... MemberExpression

    MemberExpression:
      MemberExpression . PrivateName

    Static Semantics: AllPrivateIdentifiersValid
      AllPrivateIdentifiersValid is an abstract operation which takes names as an argument.

      MemberExpression : MemberExpression . PrivateIdentifier
        1. If StringValue of PrivateIdentifier is in names, return true.
        2. Return false.

      ClassBody : ClassElementList
        1. Let newNames be the concatenation of names with PrivateBoundIdentifiers of ClassBody.
        2. Return AllPrivateIdentifiersValid of ClassElementList with the argument newNames.

    Static Semantics: Early Errors

    ScriptBody : StatementList
      It is a Syntax Error if AllPrivateIdentifiersValid of StatementList with an empty List as an argument is false unless the source code is eval code that is being processed by a direct eval.

---*/


$DONOTEVALUATE();

var C = class {
  [this.#f] = 'Test262'
};
