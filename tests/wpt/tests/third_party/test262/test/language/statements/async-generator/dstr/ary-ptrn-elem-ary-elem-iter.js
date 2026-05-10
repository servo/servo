// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-ary-elem-iter.case
// - src/dstr-binding/default/async-gen-func-decl.template
/*---
description: BindingElement with array binding pattern and initializer is not used (async generator function declaration)
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

    BindingElement : BindingPatternInitializer opt

    1. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       [...]
       e. Else,
          i. Let v be IteratorValue(next).
          [...]
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.
---*/


var callCount = 0;
async function* f([[x, y, z] = [4, 5, 6]]) {
  assert.sameValue(x, 7);
  assert.sameValue(y, 8);
  assert.sameValue(z, 9);
  callCount = callCount + 1;
};
f([[7, 8, 9]]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
