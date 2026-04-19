// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-ref-prior.case
// - src/function-forms/default/async-meth.template
/*---
description: Referencing a parameter that occurs earlier in the ParameterList (async method)
esid: sec-async-function-definitions
features: [default-parameters, async-functions]
flags: [generated, async]
info: |
    14.6 Async Function Definitions

    AsyncMethod :
     async PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }


    14.1.19 Runtime Semantics: IteratorBindingInitialization

    FormalsList : FormalsList , FormalParameter

    1. Let status be the result of performing IteratorBindingInitialization for
       FormalsList using iteratorRecord and environment as the arguments.
    2. ReturnIfAbrupt(status).
    3. Return the result of performing IteratorBindingInitialization for
       FormalParameter using iteratorRecord and environment as the arguments.

---*/
var x = 0;


var callCount = 0;
var __obj = {
  async method(x, y = x, z = y) {
    assert.sameValue(x, 3, 'first argument value');
    assert.sameValue(y, 3, 'second argument value');
    assert.sameValue(z, 3, 'third argument value');
    callCount = callCount + 1;
  }
};

// Stores a reference `ref` for case evaluation
var ref = __obj.method;

ref(3).then(() => {
    assert.sameValue(callCount, 1, 'async method invoked exactly once');
}).then($DONE, $DONE);
