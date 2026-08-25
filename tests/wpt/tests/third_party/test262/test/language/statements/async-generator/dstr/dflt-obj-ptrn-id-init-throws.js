// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-id-init-throws.case
// - src/dstr-binding/error/async-gen-func-decl-dflt.template
/*---
description: Error thrown when evaluating the initializer (async generator function declaration (default parameter))
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    6. If Initializer is present and v is undefined, then
       a. Let  defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       c. ReturnIfAbrupt(v).
---*/
function thrower() {
  throw new Test262Error();
}


async function* f({ x = thrower() } = {}) {
  
};

assert.throws(Test262Error, function() {
  f();
});
