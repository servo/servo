// This file was procedurally generated from the following sources:
// - src/function-forms/params-trailing-comma-single.case
// - src/function-forms/default/async-func-expr-named.template
/*---
description: A trailing comma should not increase the respective length, using a single parameter (async function named expression)
esid: sec-async-function-definitions
features: [async-functions]
flags: [generated, async]
info: |
    14.6 Async Function Definitions

    AsyncFunctionExpression :
      async function BindingIdentifier ( FormalParameters ) { AsyncFunctionBody }


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] : FormalParameterList[?Yield, ?Await] ,
---*/


var callCount = 0;

// Stores a reference `ref` for case evaluation
var ref;
ref = async function ref(a,) {
  assert.sameValue(a, 42);
  callCount = callCount + 1;
};

ref(42, 39).then(() => {
    assert.sameValue(callCount, 1, 'function invoked exactly once');
}).then($DONE, $DONE);

assert.sameValue(ref.length, 1, 'length is properly set');
