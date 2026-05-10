// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-ary-value-null.case
// - src/dstr-binding/error/async-gen-func-expr-dflt.template
/*---
description: Object binding pattern with "nested" array binding pattern taking the `null` value (async generator function expression (default parameter))
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


var f = async function*({ w: [x, y, z] = [4, 5, 6] } = { w: null }) {
  
};

assert.throws(TypeError, function() {
  f();
});
