// This file was procedurally generated from the following sources:
// - src/function-forms/params-trailing-comma-multiple.case
// - src/function-forms/default/async-meth.template
/*---
description: A trailing comma should not increase the respective length, using multiple parameters (async method)
esid: sec-async-function-definitions
features: [async-functions]
flags: [generated, async]
info: |
    14.6 Async Function Definitions

    AsyncMethod :
     async PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] : FormalParameterList[?Yield, ?Await] ,
---*/


var callCount = 0;
var __obj = {
  async method(a, b,) {
    assert.sameValue(a, 42);
    assert.sameValue(b, 39);
    callCount = callCount + 1;
  }
};

// Stores a reference `ref` for case evaluation
var ref = __obj.method;

ref(42, 39, 1).then(() => {
    assert.sameValue(callCount, 1, 'async method invoked exactly once');
}).then($DONE, $DONE);

assert.sameValue(ref.length, 2, 'length is properly set');
