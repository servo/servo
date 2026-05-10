// This file was procedurally generated from the following sources:
// - src/function-forms/eval-var-scope-syntax-err.case
// - src/function-forms/error-no-strict/async-arrow-function.template
/*---
description: sloppy direct eval in params introduces var (async arrow function expression in sloppy code)
esid: sec-async-arrow-function-definitions
features: [default-parameters, async-functions]
flags: [generated, async, noStrict]
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


    
    Runtime Semantics: IteratorBindingInitialization
    FormalParameter : BindingElement

    1. Return the result of performing IteratorBindingInitialization for BindingElement with arguments iteratorRecord and environment.

---*/


var callCount = 0;
var f;
f = async (a = eval("var a = 42")) => {
  
  callCount = callCount + 1;
};

f()
  .then(_ => {
    throw new Test262Error('function should not be resolved');
  }, error => assert.sameValue(error.constructor, SyntaxError))
  .then(() => {
    assert.sameValue(callCount, 0, 'function body is not evaluated');
  }, $DONE)
  .then($DONE, $DONE);
