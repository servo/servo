// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-init-throws.case
// - src/dstr-binding/error/async-gen-func-expr.template
/*---
description: Error thrown when evaluating the initializer (async generator function expression)
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
---*/
function thrower() {
  throw new Test262Error();
}


var f;
f = async function*({ x: y = thrower() }) {
  
};

assert.throws(Test262Error, function() {
  f({});
});
