// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-init-null.case
// - src/dstr-binding/error/async-gen-func-named-expr-dflt.template
/*---
description: Value specifed for object binding pattern must be object coercible (null) (async generator named function expression (default parameter))
esid: sec-asyncgenerator-definitions-evaluation
features: [async-iteration]
flags: [generated]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
        [...]

    Runtime Semantics: BindingInitialization

    ObjectBindingPattern : { }

    1. Return NormalCompletion(empty).
---*/


var f;
f = async function* h({} = null) {
  
};

assert.throws(TypeError, function() {
  f();
});
