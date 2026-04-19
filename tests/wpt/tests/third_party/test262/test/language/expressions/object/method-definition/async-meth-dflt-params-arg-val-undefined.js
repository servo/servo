// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-arg-val-undefined.case
// - src/function-forms/default/async-meth.template
/*---
description: Use of initializer when argument value is `undefined` (async method)
esid: sec-async-function-definitions
features: [default-parameters, async-functions]
flags: [generated, async]
info: |
    14.6 Async Function Definitions

    AsyncMethod :
     async PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }


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
var __obj = {
  async method(fromLiteral = 23, fromExpr = 45, fromHole = 99) {
    assert.sameValue(fromLiteral, 23);
    assert.sameValue(fromExpr, 45);
    assert.sameValue(fromHole, 99);
    callCount = callCount + 1;
  }
};

// Stores a reference `ref` for case evaluation
var ref = __obj.method;

ref(undefined, void 0).then(() => {
    assert.sameValue(callCount, 1, 'async method invoked exactly once');
}).then($DONE, $DONE);
