// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-getiter-async-get-abrupt.case
// - src/async-generators/default/async-expression-named.template
/*---
description: Abrupt completion while getting [Symbol.asyncIterator] (Named async generator expression)
esid: prod-AsyncGeneratorExpression
features: [Symbol.iterator, Symbol.asyncIterator, async-iteration]
flags: [generated, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorExpression :
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }


    YieldExpression: yield * AssignmentExpression

    1. Let exprRef be the result of evaluating AssignmentExpression.
    2. Let value be ? GetValue(exprRef).
    3. Let generatorKind be ! GetGeneratorKind().
    4. Let iterator be ? GetIterator(value, generatorKind).
    ...

    GetIterator ( obj [ , hint ] )

    ...
    3. If hint is async,
      a. Set method to ? GetMethod(obj, @@asyncIterator).
    ...

    GetMethod ( V, P )

    ...
    2. Let func be ? GetV(V, P).
    ...

---*/
var reason = {};
var obj = {
  get [Symbol.iterator]() {
    throw new Test262Error('it should not get Symbol.iterator');
  },
  get [Symbol.asyncIterator]() {
    throw reason;
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
