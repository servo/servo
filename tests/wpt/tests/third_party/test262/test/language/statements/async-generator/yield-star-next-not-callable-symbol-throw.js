// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-next-not-callable-symbol-throw.case
// - src/async-generators/default/async-declaration.template
/*---
description: Not-callable next value in a yield star position - symbol (Async generator Function declaration)
esid: prod-AsyncGeneratorDeclaration
features: [Symbol.iterator, Symbol.asyncIterator, async-iteration]
flags: [generated, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorDeclaration:
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
var obj = {
  get [Symbol.iterator]() {
    throw new Test262Error('it should not get Symbol.iterator');
  },
  [Symbol.asyncIterator]() {
    return {
      next: Symbol('oi')
    };
  }
};



var callCount = 0;

async function *gen() {
  callCount += 1;
  yield* obj;
    throw new Test262Error('abrupt completion closes iter');

}

var iter = gen();

iter.next().then(() => {
  throw new Test262Error('Promise incorrectly fulfilled.');
}, v => {
  assert.sameValue(v.constructor, TypeError, "TypeError");

  iter.next().then(({ done, value }) => {
    assert.sameValue(done, true, 'the iterator is completed');
    assert.sameValue(value, undefined, 'value is undefined');
  }).then($DONE, $DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);
