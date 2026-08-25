// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-next-then-non-callable-undefined-fulfillpromise.case
// - src/async-generators/default/async-expression-named.template
/*---
description: FulfillPromise if next().then is not-callable (undefined) (Named async generator expression)
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
        iii. If generatorKind is async, then set innerResult to
             ? Await(innerResult).
        iv. If Type(innerResult) is not Object, throw a TypeError exception.
    ...

    Await

    ...
    2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
    3. Perform ! Call(promiseCapability.[[Resolve]], undefined, « promise »).
    ...

    Promise Resolve Functions

    ...
    7. If Type(resolution) is not Object, then
      a. Return FulfillPromise(promise, resolution).
    8. Let then be Get(resolution, "then").
    ...
    11. If IsCallable(thenAction) is false, then
      a. Return FulfillPromise(promise, resolution).
    ...

---*/
var obj = {
  get [Symbol.iterator]() {
    throw new Test262Error('it should not get Symbol.iterator');
  },
  [Symbol.asyncIterator]() {
    return {
      next() {
        return {
          then: undefined,
          value: 42,
          done: false
        }
      }
    };
  }
};



var callCount = 0;

var gen = async function *g() {
  callCount += 1;
  yield* obj;
    throw new Test262Error('completion closes iter');

};

var iter = gen();

iter.next().then(({ value, done }) => {
  assert.sameValue(value, 42);
  assert.sameValue(done, false);
}).then($DONE, $DONE);

assert.sameValue(callCount, 1);
