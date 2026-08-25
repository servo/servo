// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-init-fn-name-gen.case
// - src/dstr-binding/default/async-gen-meth.template
/*---
description: SingleNameBinding assigns name to "anonymous" generator functions (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [generators, async-iteration]
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

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    6. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       c. ReturnIfAbrupt(v).
       d. If IsAnonymousFunctionDefinition(Initializer) is true, then
          [...]
    7. If environment is undefined, return PutValue(lhs, v).
    8. Return InitializeReferencedBinding(lhs, v).

---*/


var callCount = 0;
var obj = {
  async *method([gen = function* () {}, xGen = function* x() {}]) {
    assert.sameValue(gen.name, 'gen');
    assert.notSameValue(xGen.name, 'xGen');
    callCount = callCount + 1;
  }
};

obj.method([]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
