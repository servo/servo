// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-id-get-value-err.case
// - src/dstr-binding/error/async-gen-func-expr.template
/*---
description: Error thrown when accessing the corresponding property of the value object (async generator function expression)
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


var f;
f = async function*({ poisoned }) {
  
};

assert.throws(Test262Error, function() {
  f(poisonedProperty);
});
