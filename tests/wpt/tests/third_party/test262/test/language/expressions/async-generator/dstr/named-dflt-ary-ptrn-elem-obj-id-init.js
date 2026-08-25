// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-obj-id-init.case
// - src/dstr-binding/default/async-gen-func-named-expr-dflt.template
/*---
description: BindingElement with object binding pattern and initializer is used (async generator named function expression (default parameter))
esid: sec-asyncgenerator-definitions-evaluation
features: [async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
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
f = async function* h([{ x, y, z } = { x: 44, y: 55, z: 66 }] = []) {
  assert.sameValue(x, 44);
  assert.sameValue(y, 55);
  assert.sameValue(z, 66);
  callCount = callCount + 1;
};

f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
