// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id.case
// - src/dstr-binding/default/async-gen-func-expr.template
/*---
description: Lone rest element (async generator function expression)
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
    BindingRestElement : ... BindingIdentifier
    [...] 3. Let A be ArrayCreate(0). [...] 5. Repeat
       [...]
       f. Let status be CreateDataProperty(A, ToString (n), nextValue).
       [...]
---*/
var values = [1, 2, 3];


var callCount = 0;
var f;
f = async function*([...x]) {
  assert(Array.isArray(x));
  assert.sameValue(x.length, 3);
  assert.sameValue(x[0], 1);
  assert.sameValue(x[1], 2);
  assert.sameValue(x[2], 3);
  assert.notSameValue(x, values);
  callCount = callCount + 1;
};

f(values).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
