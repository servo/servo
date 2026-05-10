// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-duplicates.case
// - src/function-forms/syntax/async-arrow-function.template
/*---
description: It is a Syntax Error if BoundNames of FormalParameters contains any duplicate elements. (async arrow function expression)
esid: sec-async-arrow-function-definitions
features: [default-parameters]
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

    14.1.2 Static Semantics: Early Errors

    StrictFormalParameters : FormalParameters

    - It is a Syntax Error if BoundNames of FormalParameters contains any
      duplicate elements.

    FormalParameters : FormalParameterList

    - It is a Syntax Error if IsSimpleParameterList of FormalParameterList is
      false and BoundNames of FormalParameterList contains any duplicate
      elements.

---*/
$DONOTEVALUATE();


(async (x = 0, x) => {
  
});
