// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-init-null.case
// - src/dstr-binding/error/async-gen-meth.template
/*---
description: Value specifed for object binding pattern must be object coercible (null) (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [async-iteration]
flags: [generated]
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

    Runtime Semantics: BindingInitialization

    ObjectBindingPattern : { }

    1. Return NormalCompletion(empty).
---*/


var obj = {
  async *method({}) {
    
  }
};

assert.throws(TypeError, function() {
  obj.method(null);
});
