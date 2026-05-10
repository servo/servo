// This file was procedurally generated from the following sources:
// - src/async-generators/yield-as-identifier-reference-escaped.case
// - src/async-generators/syntax/async-expression-named.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Named async generator expression)
esid: prod-AsyncGeneratorExpression
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Generator Function Definitions

    AsyncGeneratorExpression :
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();


var gen = async function *g() {
  void yi\u0065ld;
};
