// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-init-unresolvable.case
// - src/dstr-binding/error/async-gen-func-expr.template
/*---
description: Destructuring initializer is an unresolvable reference (async generator function expression)
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    BindingElement : BindingPattern Initializeropt

    [...]
    3. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       c. ReturnIfAbrupt(v).

    6.2.3.1 GetValue (V)

    1. ReturnIfAbrupt(V).
    2. If Type(V) is not Reference, return V.
    3. Let base be GetBase(V).
    4. If IsUnresolvableReference(V), throw a ReferenceError exception.
---*/


var f;
f = async function*({ x: y = unresolvableReference }) {
  
};

assert.throws(ReferenceError, function() {
  f({});
});
