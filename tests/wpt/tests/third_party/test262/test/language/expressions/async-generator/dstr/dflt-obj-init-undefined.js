// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-init-undefined.case
// - src/dstr-binding/error/async-gen-func-expr-dflt.template
/*---
description: Value specifed for object binding pattern must be object coercible (undefined) (async generator function expression (default parameter))
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


var f = async function*({} = undefined) {
  
};

assert.throws(TypeError, function() {
  f();
});
