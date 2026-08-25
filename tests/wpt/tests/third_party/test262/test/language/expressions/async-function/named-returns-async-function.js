// This file was procedurally generated from the following sources:
// - src/async-functions/returns-async-function.case
// - src/async-functions/evaluation/async-expression-named.template
/*---
description: Async function returns an async function. (Named async function expression)
esid: prod-AsyncFunctionExpression
features: [async-functions]
flags: [generated, async]
info: |
    Async Function Definitions

    AsyncFunctionExpression :
      async [no LineTerminator here] function BindingIdentifier ( FormalParameters ) { AsyncFunctionBody }

---*/
let count = 0;


var asyncFn = async function asyncFn(x) {
  return async function() { return x; };
};

asyncFn(1).then(retFn => {
  count++;
  return retFn();
}).then(result => {
  assert.sameValue(result, 1);
  assert.sameValue(count, 1);
}).then($DONE, $DONE);
