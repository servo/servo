// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-exhausted.case
// - src/dstr-binding/default/async-gen-meth.template
/*---
description: RestElement applied to an exhausted iterator (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [Symbol.iterator, async-iteration]
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


    13.3.3.6 Runtime Semantics: IteratorBindingInitialization
    BindingRestElement : ... BindingIdentifier
    1. Let lhs be ResolveBinding(StringValue of BindingIdentifier,
       environment).
    2. ReturnIfAbrupt(lhs). 3. Let A be ArrayCreate(0). 4. Let n=0. 5. Repeat,
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. If environment is undefined, return PutValue(lhs, A).
          ii. Return InitializeReferencedBinding(lhs, A).

---*/


var callCount = 0;
var obj = {
  async *method([, , ...x]) {
    assert(Array.isArray(x));
    assert.sameValue(x.length, 0);
    callCount = callCount + 1;
  }
};

obj.method([1, 2]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
