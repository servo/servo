// This file was procedurally generated from the following sources:
// - src/function-forms/params-trailing-comma-single.case
// - src/function-forms/default/async-gen-named-func-expr.template
/*---
description: A trailing comma should not increase the respective length, using a single parameter (async generator named function expression)
esid: sec-asyncgenerator-definitions-evaluation
features: [async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
        [...]


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] : FormalParameterList[?Yield, ?Await] ,
---*/


var callCount = 0;
// Stores a reference `ref` for case evaluation
var ref;
ref = async function* g(a,) {
  assert.sameValue(a, 42);
  callCount = callCount + 1;
};

ref(42, 39).next().then(() => {
    assert.sameValue(callCount, 1, 'generator function invoked exactly once');
}).then($DONE, $DONE);

assert.sameValue(ref.length, 1, 'length is properly set');
