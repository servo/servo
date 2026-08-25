// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-init-null.case
// - src/dstr-binding/error/async-gen-func-decl-dflt.template
/*---
description: Value specifed for object binding pattern must be object coercible (null) (async generator function declaration (default parameter))
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


async function* f({} = null) {
  
};

assert.throws(TypeError, function() {
  f();
});
