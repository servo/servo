// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-eval-err.case
// - src/dstr-binding/error/async-gen-func-decl.template
/*---
description: Evaluation of property name returns an abrupt completion (async generator function declaration)
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

    BindingProperty : PropertyName : BindingElement

    1. Let P be the result of evaluating PropertyName
    2. ReturnIfAbrupt(P).
---*/
function thrower() {
  throw new Test262Error();
}


async function* f({ [thrower()]: x }) {
  
};

assert.throws(Test262Error, function() {
  f({});
});
