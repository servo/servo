// This file was procedurally generated from the following sources:
// - src/function-forms/rest-params-trailing-comma-early-error.case
// - src/function-forms/syntax/async-arrow-function.template
/*---
description: It's a syntax error if a FunctionRestParameter is followed by a trailing comma (async arrow function expression)
esid: sec-async-arrow-function-definitions
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
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

    FormalParameters[Yield, Await] :
        [empty]
        FunctionRestParameter[?Yield, ?Await]
        FormalParameterList[?Yield, ?Await]
        FormalParameterList[?Yield, ?Await] ,
        FormalParameterList[?Yield, ?Await] , FunctionRestParameter[?Yield, ?Await]
---*/
$DONOTEVALUATE();


(async (...a,) => {
  
});
