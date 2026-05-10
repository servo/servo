// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-ary-empty-iter.case
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
var initCount = 0;


var callCount = 0;
async function* f([[] = function() { initCount += 1; }()]) {
  assert.sameValue(initCount, 0);
  callCount = callCount + 1;
};
f([[23]]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
