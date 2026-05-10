// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-ref-prior.case
// - src/function-forms/default/async-gen-meth.template
/*---
description: Referencing a parameter that occurs earlier in the ParameterList (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [default-parameters, async-iteration]
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
var obj = {
  async *method(x, y = x, z = y) {
    assert.sameValue(x, 3, 'first argument value');
    assert.sameValue(y, 3, 'second argument value');
    assert.sameValue(z, 3, 'third argument value');
    callCount = callCount + 1;
  }
};

// Stores a reference `ref` for case evaluation
var ref = obj.method;

ref(3).next().then(() => {
    assert.sameValue(callCount, 1, 'generator method invoked exactly once');
}).then($DONE, $DONE);
