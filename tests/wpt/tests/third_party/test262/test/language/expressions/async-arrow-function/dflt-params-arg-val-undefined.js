// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-arg-val-undefined.case
// - src/function-forms/default/async-arrow-function.template
/*---
description: Use of initializer when argument value is `undefined` (async arrow function expression)
esid: sec-async-arrow-function-definitions
features: [default-parameters, async-functions]
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


    14.1.19 Runtime Semantics: IteratorBindingInitialization

    FormalsList : FormalsList , FormalParameter

    [...]
    23. Let iteratorRecord be Record {[[Iterator]]:
        CreateListIterator(argumentsList), [[Done]]: false}.
    24. If hasDuplicates is true, then
        [...]
    25. Else,
        a. Perform ? IteratorBindingInitialization for formals with
           iteratorRecord and env as arguments.
    [...]

---*/


var callCount = 0;

// Stores a reference `ref` for case evaluation
var ref = async (fromLiteral = 23, fromExpr = 45, fromHole = 99) => {
  assert.sameValue(fromLiteral, 23);
  assert.sameValue(fromExpr, 45);
  assert.sameValue(fromHole, 99);
  callCount = callCount + 1;
};

ref(undefined, void 0).then(() => {
  assert.sameValue(callCount, 1, 'async arrow function invoked exactly once')
}).then($DONE, $DONE);
