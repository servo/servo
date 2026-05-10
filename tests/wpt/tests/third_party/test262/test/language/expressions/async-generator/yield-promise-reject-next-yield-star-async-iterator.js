// This file was procedurally generated from the following sources:
// - src/async-generators/yield-promise-reject-next-yield-star-async-iterator.case
// - src/async-generators/default/async-expression.template
/*---
description: yield * (async iterator) is treated as throw value (Unnamed async generator expression)
esid: prod-AsyncGeneratorExpression
features: [async-iteration]
flags: [generated, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorExpression :
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }

---*/
let error = new Error();
async function * readFile() {
  yield Promise.reject(error);
  yield "unreachable";
}


var callCount = 0;

var gen = async function *() {
  callCount += 1;
  yield * readFile();

};

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
}, $DONE).catch($DONE);

assert.sameValue(callCount, 1);
