// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-trailing-comma.case
// - src/dstr-binding/default/async-gen-func-decl.template
/*---
description: Trailing comma is allowed following BindingPropertyList (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
        [...]


    13.3.3 Destructuring Binding Patterns

    ObjectBindingPattern[Yield] :
        { }
        { BindingPropertyList[?Yield] }
        { BindingPropertyList[?Yield] , }
---*/


var callCount = 0;
async function* f({ x: y, }) {
  assert.sameValue(y, 23);

  assert.throws(ReferenceError, function() {
    x;
  });
  callCount = callCount + 1;
};
f({ x: 23 }).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
