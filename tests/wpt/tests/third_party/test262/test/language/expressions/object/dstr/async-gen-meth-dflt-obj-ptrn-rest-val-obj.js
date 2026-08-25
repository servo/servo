// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-val-obj.case
// - src/dstr-binding/default/async-gen-method-dflt.template
/*---
description: Rest object contains just unextracted data (async generator method (default parameter))
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [object-rest, async-iteration]
flags: [generated, async]
includes: [propertyHelper.js]
info: |
    AsyncGeneratorMethod :
        async [no LineTerminator here] * PropertyName ( UniqueFormalParameters )
            { AsyncGeneratorBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this AsyncGeneratorMethod is strict mode code, let strict be true.
       Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be ! AsyncGeneratorFunctionCreate(Method, UniqueFormalParameters,
       AsyncGeneratorBody, scope, strict).
    [...]

---*/


var callCount = 0;
var obj = {
  async *method({a, b, ...rest} = {x: 1, y: 2, a: 5, b: 3}) {
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
  }
};

obj.method().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
