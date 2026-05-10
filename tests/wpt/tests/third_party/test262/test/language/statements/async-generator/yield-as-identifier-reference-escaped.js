// This file was procedurally generated from the following sources:
// - src/async-generators/yield-as-identifier-reference-escaped.case
// - src/async-generators/syntax/async-declaration.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Async generator Function declaration)
esid: prod-AsyncGeneratorDeclaration
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Generator Function Definitions

    AsyncGeneratorDeclaration:
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();


async function *gen() {
  void yi\u0065ld;
}
