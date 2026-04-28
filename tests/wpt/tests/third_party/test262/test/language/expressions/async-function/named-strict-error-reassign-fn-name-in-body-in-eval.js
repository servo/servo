// This file was procedurally generated from the following sources:
// - src/function-forms/reassign-fn-name-in-body-in-eval.case
// - src/function-forms/expr-named/async-func-expr-named-strict-error.template
/*---
description: Reassignment of function name is silently ignored in non-strict mode code. (async function named expression in strict mode code)
esid: sec-async-function-definitions
features: [async-functions]
flags: [generated, async, onlyStrict]
includes: [asyncHelpers.js]
info: |
    Async Function Definitions

    AsyncFunctionExpression :
      async function BindingIdentifier ( FormalParameters ) { AsyncFunctionBody }

---*/

// increment callCount in case "body"
let callCount = 0;
let ref = async function BindingIdentifier() {
  callCount++;
  eval("BindingIdentifier = 1");
  return BindingIdentifier;
};

asyncTest(async () => {
  let catchCount = 0;
  try {
    await ref()
  } catch (error) {
    catchCount++;
    assert(error instanceof TypeError);
  }
  assert.sameValue(catchCount, 1);
  assert.sameValue(callCount, 1);
});

