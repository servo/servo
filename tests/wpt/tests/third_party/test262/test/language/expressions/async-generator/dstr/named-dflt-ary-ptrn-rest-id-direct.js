// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-direct.case
// - src/dstr-binding/default/async-gen-func-named-expr-dflt.template
/*---
description: Lone rest element (direct binding) (async generator named function expression (default parameter))
esid: sec-asyncgenerator-definitions-evaluation
features: [async-iteration]
flags: [generated, async]
includes: [compareArray.js]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
        [...]


    Runtime Semantics: IteratorBindingInitialization

    BindingRestElement : ... BindingIdentifier

    [...]
    2. Let A be ! ArrayCreate(0).
    3. Let n be 0.
    4. Repeat,
        [...]
        f. Perform ! CreateDataPropertyOrThrow(A, ! ToString(n), nextValue).
        g. Set n to n + 1.

---*/


var callCount = 0;
var f;
f = async function* h([...x] = [1]) {
  assert(Array.isArray(x));
  assert.compareArray(x, [1]);
  callCount = callCount + 1;
};

f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
