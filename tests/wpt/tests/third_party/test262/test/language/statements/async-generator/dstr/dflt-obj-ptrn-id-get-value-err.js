// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-id-get-value-err.case
// - src/dstr-binding/error/async-gen-func-decl-dflt.template
/*---
description: Error thrown when accessing the corresponding property of the value object (async generator function declaration (default parameter))
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
    4. Let v be GetV(value, propertyName).
    5. ReturnIfAbrupt(v).
---*/
var poisonedProperty = Object.defineProperty({}, 'poisoned', {
  get: function() {
    throw new Test262Error();
  }
});


async function* f({ poisoned } = poisonedProperty) {
  
};

assert.throws(Test262Error, function() {
  f();
});
