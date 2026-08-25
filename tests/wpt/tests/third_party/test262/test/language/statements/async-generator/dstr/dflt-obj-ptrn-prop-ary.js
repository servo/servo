// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-ary.case
// - src/dstr-binding/default/async-gen-func-decl-dflt.template
/*---
description: Object binding pattern with "nested" array binding pattern not using initializer (async generator function declaration (default parameter))
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


    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    [...]
    3. If Initializer is present and v is undefined, then
       [...]
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/


var callCount = 0;
async function* f({ w: [x, y, z] = [4, 5, 6] } = { w: [7, undefined, ] }) {
  assert.sameValue(x, 7);
  assert.sameValue(y, undefined);
  assert.sameValue(z, undefined);

  assert.throws(ReferenceError, function() {
    w;
  });
  callCount = callCount + 1;
};
f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
