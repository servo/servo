// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-list-err.case
// - src/dstr-binding/error/async-gen-func-decl-dflt.template
/*---
description: Binding property list evaluation is interrupted by an abrupt completion (async generator function declaration (default parameter))
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


async function* f({ a, b = thrower(), c = ++initCount } = {}) {
  
};

assert.throws(Test262Error, function() {
  f();
});

assert.sameValue(initCount, 0);
