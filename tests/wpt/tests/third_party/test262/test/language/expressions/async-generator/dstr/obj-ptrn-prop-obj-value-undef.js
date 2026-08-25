// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-obj-value-undef.case
// - src/dstr-binding/error/async-gen-func-expr.template
/*---
description: Object binding pattern with "nested" object binding pattern taking the `null` value (async generator function expression)
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

    [...]
    3. If Initializer is present and v is undefined, then
       [...]
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/


var f;
f = async function*({ w: { x, y, z } = undefined }) {
  
};

assert.throws(TypeError, function() {
  f({ });
});
