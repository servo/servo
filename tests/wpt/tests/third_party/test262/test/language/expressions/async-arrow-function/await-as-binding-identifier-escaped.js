// This file was procedurally generated from the following sources:
// - src/async-functions/await-as-binding-identifier-escaped.case
// - src/async-functions/syntax/async-arrow.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as a binding identifier. (Async arrow function)
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


    BindingIdentifier : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


async () => {
  var \u0061wait;
}
