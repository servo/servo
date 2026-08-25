// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-obj-prop-id.case
// - src/dstr-binding/default/async-gen-func-expr-dflt.template
/*---
description: Rest element containing an object binding pattern (async generator function expression (default parameter))
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
var f;
f = async function*([...{ 0: v, 1: w, 2: x, 3: y, length: z }] = [7, 8, 9]) {
  assert.sameValue(v, 7);
  assert.sameValue(w, 8);
  assert.sameValue(x, 9);
  assert.sameValue(y, undefined);
  assert.sameValue(z, 3);

  assert.sameValue(length, "outer", "the length prop is not set as a binding name");
  callCount = callCount + 1;
};

f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
