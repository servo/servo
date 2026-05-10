// This file was procedurally generated from the following sources:
// - src/async-functions/await-as-identifier-reference.case
// - src/async-functions/syntax/async-arrow.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Async arrow function)
esid: prod-AsyncArrowFunction
features: [async-functions]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Arrow Function Definitions

    AsyncArrowFunction[In, Yield, Await]:
      async [no LineTerminator here] AsyncArrowBindingIdentifier[?Yield] [no LineTerminator here] => AsyncConciseBody[?In]
      CoverCallExpressionAndAsyncArrowHead[?Yield, ?Await] [no LineTerminator here] => AsyncConciseBody[?In]

    AsyncConciseBody[In]:
      { AsyncFunctionBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


async () => {
  void await;
}
