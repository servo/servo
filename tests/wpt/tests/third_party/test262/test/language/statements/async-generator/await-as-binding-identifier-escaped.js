// This file was procedurally generated from the following sources:
// - src/async-generators/await-as-binding-identifier-escaped.case
// - src/async-generators/syntax/async-declaration.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as a binding identifier. (Async generator Function declaration)
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


    BindingIdentifier : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


async function *gen() {
  var \u0061wait;
}
