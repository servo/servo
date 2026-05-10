// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-exhausted.case
// - src/dstr-binding/default/async-gen-func-decl.template
/*---
description: RestElement applied to an exhausted iterator (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [Symbol.iterator, async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
        [...]


    13.3.3.6 Runtime Semantics: IteratorBindingInitialization
    BindingRestElement : ... BindingIdentifier
    1. Let lhs be ResolveBinding(StringValue of BindingIdentifier,
       environment).
    2. ReturnIfAbrupt(lhs). 3. Let A be ArrayCreate(0). 4. Let n=0. 5. Repeat,
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. If environment is undefined, return PutValue(lhs, A).
          ii. Return InitializeReferencedBinding(lhs, A).

---*/


var callCount = 0;
async function* f([, , ...x]) {
  assert(Array.isArray(x));
  assert.sameValue(x.length, 0);
  callCount = callCount + 1;
};
f([1, 2]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
