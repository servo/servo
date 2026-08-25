// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-empty.case
// - src/dstr-binding/default/async-gen-meth.template
/*---
description: No iteration occurs for an "empty" array binding pattern (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [generators, async-iteration]
flags: [generated, async]
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


    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    ArrayBindingPattern : [ ]

    1. Return NormalCompletion(empty).

---*/
var iterations = 0;
var iter = function*() {
  iterations += 1;
}();


var callCount = 0;
var obj = {
  async *method([]) {
    assert.sameValue(iterations, 0);
    callCount = callCount + 1;
  }
};

obj.method(iter).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
