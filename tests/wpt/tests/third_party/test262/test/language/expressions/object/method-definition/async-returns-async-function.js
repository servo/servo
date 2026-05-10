// This file was procedurally generated from the following sources:
// - src/async-functions/returns-async-function.case
// - src/async-functions/evaluation/async-obj-method.template
/*---
description: Async function returns an async function. (Async method)
esid: prod-AsyncMethod
features: [async-functions]
flags: [generated, async]
info: |
    Async Function Definitions

    AsyncMethod :
      async [no LineTerminator here] PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }

---*/
let count = 0;


var obj = {
  async method(x) {
    return async function() { return x; };
  }
};
// Stores a reference `asyncFn` for case evaluation
let asyncFn = obj.method;

asyncFn(1).then(retFn => {
  count++;
  return retFn();
}).then(result => {
  assert.sameValue(result, 1);
  assert.sameValue(count, 1);
}).then($DONE, $DONE);
