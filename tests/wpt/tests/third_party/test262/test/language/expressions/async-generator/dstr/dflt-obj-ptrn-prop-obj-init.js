// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-obj-init.case
// - src/dstr-binding/default/async-gen-func-expr-dflt.template
/*---
description: Object binding pattern with "nested" object binding pattern using initializer (async generator function expression (default parameter))
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

    [...]
    3. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       c. ReturnIfAbrupt(v).
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/


var callCount = 0;
var f;
f = async function*({ w: { x, y, z } = { x: 4, y: 5, z: 6 } } = { w: undefined }) {
  assert.sameValue(x, 4);
  assert.sameValue(y, 5);
  assert.sameValue(z, 6);

  assert.throws(ReferenceError, function() {
    w;
  });
  callCount = callCount + 1;
};

f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
