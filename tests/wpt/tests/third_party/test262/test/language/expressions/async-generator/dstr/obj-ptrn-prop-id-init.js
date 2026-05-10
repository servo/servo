// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-init.case
// - src/dstr-binding/default/async-gen-func-expr.template
/*---
description: Binding as specified via property name, identifier, and initializer (async generator function expression)
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


    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    8. Return InitializeReferencedBinding(lhs, v).
---*/


var callCount = 0;
var f;
f = async function*({ x: y = 33 }) {
  assert.sameValue(y, 33);
  assert.throws(ReferenceError, function() {
    x;
  });
  callCount = callCount + 1;
};

f({ }).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
