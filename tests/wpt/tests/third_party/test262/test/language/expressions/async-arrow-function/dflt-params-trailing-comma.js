// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-trailing-comma.case
// - src/function-forms/default/async-arrow-function.template
/*---
description: A trailing comma should not increase the respective length, using default parameters (async arrow function expression)
esid: sec-async-arrow-function-definitions
features: [async-functions]
flags: [generated, async]
info: |
    14.7 Async Arrow Function Definitions

    AsyncArrowFunction :
      ...
      CoverCallExpressionAndAsyncArrowHead => AsyncConciseBody

    AsyncConciseBody :
      { AsyncFunctionBody }

    ...

    Supplemental Syntax

    When processing an instance of the production AsyncArrowFunction :
    CoverCallExpressionAndAsyncArrowHead => AsyncConciseBody the interpretation of
    CoverCallExpressionAndAsyncArrowHead is refined using the following grammar:

    AsyncArrowHead :
      async ArrowFormalParameters


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] : FormalParameterList[?Yield, ?Await] ,
---*/


var callCount = 0;

// Stores a reference `ref` for case evaluation
var ref = async (a, b = 39,) => {
  assert.sameValue(a, 42);
  assert.sameValue(b, 39);
  callCount = callCount + 1;
};

ref(42, undefined, 1).then(() => {
  assert.sameValue(callCount, 1, 'async arrow function invoked exactly once')
}).then($DONE, $DONE);

assert.sameValue(ref.length, 1, 'length is properly set');
