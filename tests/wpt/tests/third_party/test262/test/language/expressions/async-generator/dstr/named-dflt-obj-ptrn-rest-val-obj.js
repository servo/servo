// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-val-obj.case
// - src/dstr-binding/default/async-gen-func-named-expr-dflt.template
/*---
description: Rest object contains just unextracted data (async generator named function expression (default parameter))
esid: sec-asyncgenerator-definitions-evaluation
features: [object-rest, async-iteration]
flags: [generated, async]
includes: [propertyHelper.js]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
        [...]

---*/


var callCount = 0;
var f;
f = async function* h({a, b, ...rest} = {x: 1, y: 2, a: 5, b: 3}) {
  assert.sameValue(rest.a, undefined);
  assert.sameValue(rest.b, undefined);

  verifyProperty(rest, "x", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 1
  });

  verifyProperty(rest, "y", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });
  callCount = callCount + 1;
};

f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
