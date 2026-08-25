// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-obj-prop-id.case
// - src/dstr-binding/default/async-gen-func-decl.template
/*---
description: Rest element containing an object binding pattern (async generator function declaration)
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


    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    BindingRestElement : ... BindingPattern

    1. Let A be ArrayCreate(0).
    [...]
    3. Repeat
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. Return the result of performing BindingInitialization of
             BindingPattern with A and environment as the arguments.
       [...]
---*/
let length = "outer";


var callCount = 0;
async function* f([...{ 0: v, 1: w, 2: x, 3: y, length: z }]) {
  assert.sameValue(v, 7);
  assert.sameValue(w, 8);
  assert.sameValue(x, 9);
  assert.sameValue(y, undefined);
  assert.sameValue(z, 3);

  assert.sameValue(length, "outer", "the length prop is not set as a binding name");
  callCount = callCount + 1;
};
f([7, 8, 9]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
