// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-init-hole.case
// - src/dstr-binding/default/async-gen-func-expr-dflt.template
/*---
description: Destructuring initializer with a "hole" (async generator function expression (default parameter))
esid: sec-asyncgenerator-definitions-evaluation
features: [async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * ( FormalParameters ) {
        AsyncGeneratorBody }

        [...]
        3. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, scope, strict).
        [...]


    13.3.3.6 Runtime Semantics: IteratorBindingInitialization
    SingleNameBinding : BindingIdentifier Initializeropt
    [...] 6. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       [...]
    7. If environment is undefined, return PutValue(lhs, v). 8. Return InitializeReferencedBinding(lhs, v).
---*/


var callCount = 0;
var f;
f = async function*([x = 23] = [,]) {
  assert.sameValue(x, 23);
  // another statement
  callCount = callCount + 1;
};

f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
