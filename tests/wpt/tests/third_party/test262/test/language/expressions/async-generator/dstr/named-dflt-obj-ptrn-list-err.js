// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-list-err.case
// - src/dstr-binding/error/async-gen-func-named-expr-dflt.template
/*---
description: Binding property list evaluation is interrupted by an abrupt completion (async generator named function expression (default parameter))
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

    BindingPropertyList : BindingPropertyList , BindingProperty

    1. Let status be the result of performing BindingInitialization for
       BindingPropertyList using value and environment as arguments.
    2. ReturnIfAbrupt(status).
---*/
var initCount = 0;
function thrower() {
  throw new Test262Error();
}


var f;
f = async function* h({ a, b = thrower(), c = ++initCount } = {}) {
  
};

assert.throws(Test262Error, function() {
  f();
});

assert.sameValue(initCount, 0);
