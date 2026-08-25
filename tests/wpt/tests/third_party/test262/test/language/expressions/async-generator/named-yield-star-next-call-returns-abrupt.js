// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-next-call-returns-abrupt.case
// - src/async-generators/default/async-expression-named.template
/*---
description: Abrupt completion while calling next (Named async generator expression)
esid: prod-AsyncGeneratorExpression
features: [Symbol.iterator, Symbol.asyncIterator, async-iteration]
flags: [generated, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorExpression :
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }


    YieldExpression: yield * AssignmentExpression
    ...
    6. Repeat
      a. If received.[[Type]] is normal, then
        ii. Let innerResult be ? Invoke(iterator, "next",
            « received.[[Value]] »).
    ...

---*/
var reason = {};
var obj = {
  get [Symbol.iterator]() {
    throw new Test262Error('it should not get Symbol.iterator');
  },
  [Symbol.asyncIterator]() {
    return {
      next() {
        throw reason;
      }
    };
  }
};



var callCount = 0;

var gen = async function *g() {
  callCount += 1;
  yield* obj;
    throw new Test262Error('abrupt completion closes iter');

};

var iter = gen();

iter.next().then(() => {
  throw new Test262Error('Promise incorrectly fulfilled.');
}, v => {
  assert.sameValue(v, reason, "reject reason");

  iter.next().then(({ done, value }) => {
    assert.sameValue(done, true, 'the iterator is completed');
    assert.sameValue(value, undefined, 'value is undefined');
  }).then($DONE, $DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);
