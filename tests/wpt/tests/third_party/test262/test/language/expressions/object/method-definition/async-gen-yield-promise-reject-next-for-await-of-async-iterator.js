// This file was procedurally generated from the following sources:
// - src/async-generators/yield-promise-reject-next-for-await-of-async-iterator.case
// - src/async-generators/default/async-obj-method.template
/*---
description: yield * [Promise.reject(value)] is treated as throw value (Async generator method)
esid: prod-AsyncGeneratorMethod
features: [async-iteration]
flags: [generated, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }

---*/
let error = new Error();
async function * readFile() {
  yield Promise.reject(error);
  yield "unreachable";
}

var callCount = 0;

var gen = {
  async *method() {
    callCount += 1;
    for await (let line of readFile()) {
      yield line;
    }
  }
}.method;

var iter = gen();

iter.next().then(() => {
  throw new Test262Error("Promise incorrectly resolved.");
}, rejectValue => {

  // yield Promise.reject(error);
  assert.sameValue(rejectValue, error);

  iter.next().then(({done, value}) => {
    // iter is closed now.
    assert.sameValue(done, true, "The value of IteratorResult.done is `true`");
    assert.sameValue(value, undefined, "The value of IteratorResult.value is `undefined`");
  }).then($DONE, $DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);
