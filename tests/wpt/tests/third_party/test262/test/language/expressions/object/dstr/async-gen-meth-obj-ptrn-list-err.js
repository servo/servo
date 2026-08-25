// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-list-err.case
// - src/dstr-binding/error/async-gen-meth.template
/*---
description: Binding property list evaluation is interrupted by an abrupt completion (async generator method)
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

    13.3.3.5 Runtime Semantics: BindingInitialization

    BindingPropertyList : BindingPropertyList , BindingProperty

    1. Let status be the result of performing BindingInitialization for
       BindingPropertyList using value and environment as arguments.
    2. ReturnIfAbrupt(status).
---*/
var initCount = 0;
function thrower() {
  throw new Test262Error();
}


var obj = {
  async *method({ a, b = thrower(), c = ++initCount }) {
    
  }
};

assert.throws(Test262Error, function() {
  obj.method({});
});

assert.sameValue(initCount, 0);
