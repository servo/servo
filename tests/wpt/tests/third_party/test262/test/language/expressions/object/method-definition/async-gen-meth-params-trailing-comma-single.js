// This file was procedurally generated from the following sources:
// - src/function-forms/params-trailing-comma-single.case
// - src/function-forms/default/async-gen-meth.template
/*---
description: A trailing comma should not increase the respective length, using a single parameter (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorMethod :
        async [no LineTerminator here] * PropertyName ( UniqueFormalParameters )
            { AsyncGeneratorBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this AsyncGeneratorMethod is strict mode code, let strict be true.
       Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be ! AsyncGeneratorFunctionCreate(Method, UniqueFormalParameters,
       AsyncGeneratorBody, scope, strict).
    [...]


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] : FormalParameterList[?Yield, ?Await] ,
---*/

var callCount = 0;
var obj = {
  async *method(a,) {
    assert.sameValue(a, 42);
    callCount = callCount + 1;
  }
};

// Stores a reference `ref` for case evaluation
var ref = obj.method;

ref(42, 39).next().then(() => {
    assert.sameValue(callCount, 1, 'generator method invoked exactly once');
}).then($DONE, $DONE);

assert.sameValue(ref.length, 1, 'length is properly set');
