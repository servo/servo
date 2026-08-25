// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-eval-err.case
// - src/dstr-binding/error/async-gen-func-named-expr-dflt.template
/*---
description: Evaluation of property name returns an abrupt completion (async generator named function expression (default parameter))
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

    13.3.3.5 Runtime Semantics: BindingInitialization

    BindingProperty : PropertyName : BindingElement

    1. Let P be the result of evaluating PropertyName
    2. ReturnIfAbrupt(P).
---*/
function thrower() {
  throw new Test262Error();
}


var f;
f = async function* h({ [thrower()]: x } = {}) {
  
};

assert.throws(Test262Error, function() {
  f();
});
