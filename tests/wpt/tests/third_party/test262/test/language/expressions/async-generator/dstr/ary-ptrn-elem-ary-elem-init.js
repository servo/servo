// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-ary-elem-init.case
// - src/dstr-binding/default/async-gen-func-expr.template
/*---
description: BindingElement with array binding pattern and initializer is used (async generator function expression)
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

    BindingElement : BindingPatternInitializer opt

    [...]
    2. If iteratorRecord.[[done]] is true, let v be undefined.
    3. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be ? GetValue(defaultValue).
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.
---*/


var callCount = 0;
var f;
f = async function*([[x, y, z] = [4, 5, 6]]) {
  assert.sameValue(x, 4);
  assert.sameValue(y, 5);
  assert.sameValue(z, 6);
  callCount = callCount + 1;
};

f([]).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
