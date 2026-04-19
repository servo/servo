// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-init-throws.case
// - src/dstr-binding/error/async-gen-func-expr-dflt.template
/*---
description: Destructuring initializer returns an abrupt completion (async generator function expression (default parameter))
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

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    6. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       c. ReturnIfAbrupt(v).
---*/


var f = async function*([x = (function() { throw new Test262Error(); })()] = [undefined]) {
  
};

assert.throws(Test262Error, function() {
  f();
});
