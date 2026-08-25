// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-ary-trailing-comma.case
// - src/dstr-binding/default/async-gen-func-expr.template
/*---
description: Trailing comma is allowed following BindingPropertyList (async generator function expression)
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


    13.3.3 Destructuring Binding Patterns

    ObjectBindingPattern[Yield] :
        { }
        { BindingPropertyList[?Yield] }
        { BindingPropertyList[?Yield] , }
---*/


var callCount = 0;
var f;
f = async function*({ x: [y], }) {
  assert.sameValue(y,45);
  callCount = callCount + 1;
};

f({ x: [45] }).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
