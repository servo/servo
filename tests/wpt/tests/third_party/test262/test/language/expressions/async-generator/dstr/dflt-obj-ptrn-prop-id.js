// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id.case
// - src/dstr-binding/default/async-gen-func-expr-dflt.template
/*---
description: Binding as specified via property name and identifier (async generator function expression (default parameter))
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
f = async function*({ x: y } = { x: 23 }) {
  assert.sameValue(y, 23);
  assert.throws(ReferenceError, function() {
    x;
  });
  callCount = callCount + 1;
};

f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
