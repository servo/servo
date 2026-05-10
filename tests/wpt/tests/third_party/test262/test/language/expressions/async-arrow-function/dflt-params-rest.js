// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-rest.case
// - src/function-forms/syntax/async-arrow-function.template
/*---
description: RestParameter does not support an initializer (async arrow function expression)
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

    14.1 Function Definitions

    Syntax

    FunctionRestParameter[Yield] :

      BindingRestElement[?Yield]

    13.3.3 Destructuring Binding Patterns

    Syntax

    BindingRestElement[Yield] :

      ...BindingIdentifier[?Yield]
      ...BindingPattern[?Yield]

---*/
$DONOTEVALUATE();


(async (...x = []) => {
  
});
