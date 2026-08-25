// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-init-undefined.case
// - src/dstr-binding/error/async-gen-func-decl.template
/*---
description: Value specifed for object binding pattern must be object coercible (undefined) (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [async-iteration]
flags: [generated]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
        [...]

    Runtime Semantics: BindingInitialization

    ObjectBindingPattern : { }

    1. Return NormalCompletion(empty).
---*/


async function* f({}) {
  
};

assert.throws(TypeError, function() {
  f(undefined);
});
