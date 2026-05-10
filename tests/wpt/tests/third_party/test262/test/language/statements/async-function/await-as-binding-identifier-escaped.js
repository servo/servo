// This file was procedurally generated from the following sources:
// - src/async-functions/await-as-binding-identifier-escaped.case
// - src/async-functions/syntax/async-declaration.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as a binding identifier. (Async function declaration)
esid: prod-AsyncFunctionDeclaration
features: [async-functions]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Function Definitions

    AsyncFunctionDeclaration:
      async [no LineTerminator here] function BindingIdentifier ( FormalParameters ) { AsyncFunctionBody }


    BindingIdentifier : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


async function asyncFn() {
  var \u0061wait;
}
