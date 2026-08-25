// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-init-undefined.case
// - src/dstr-binding/error/async-gen-func-expr.template
/*---
description: Value specifed for object binding pattern must be object coercible (undefined) (async generator function expression)
esid: sec-asyncgenerator-definitions-evaluation
features: [async-iteration]
flags: [generated]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * ( FormalParameters ) {
        AsyncGeneratorBody }

        [...]
        3. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, scope, strict).
        [...]

    Runtime Semantics: BindingInitialization

    ObjectBindingPattern : { }

    1. Return NormalCompletion(empty).
---*/


var f;
f = async function*({}) {
  
};

assert.throws(TypeError, function() {
  f(undefined);
});
