// This file was procedurally generated from the following sources:
// - src/async-functions/returns-async-arrow-returns-newtarget.case
// - src/async-functions/evaluation/async-declaration.template
/*---
description: Async function returns an async function. (Async function declaration)
esid: prod-AsyncFunctionDeclaration
features: [async-functions]
flags: [generated, async]
info: |
    Async Function Definitions

    AsyncFunctionDeclaration:
      async [no LineTerminator here] function BindingIdentifier ( FormalParameters ) { AsyncFunctionBody }

---*/
let count = 0;


async function asyncFn(x) {
  return async () => new.target;
}

asyncFn().then(retFn => {
  count++;
  return retFn();
}).then(result => {
  assert.sameValue(result, undefined);
  assert.sameValue(count, 1);
}).then($DONE, $DONE);
