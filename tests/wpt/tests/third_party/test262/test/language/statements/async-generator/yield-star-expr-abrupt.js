// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-expr-abrupt.case
// - src/async-generators/default/async-declaration.template
/*---
description: Abrupt completion while getting yield* operand (Async generator Function declaration)
esid: prod-AsyncGeneratorDeclaration
features: [async-iteration]
flags: [generated, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorDeclaration:
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }


    YieldExpression: yield * AssignmentExpression

    1. Let exprRef be the result of evaluating AssignmentExpression.
    2. Let value be ? GetValue(exprRef).
    ...

---*/
var obj = {};
var abrupt = function() {
  throw obj;
};



var callCount = 0;

async function *gen() {
  callCount += 1;
  yield* abrupt();
    throw new Test262Error('abrupt completion closes iter');

}

var iter = gen();

iter.next().then(() => {
  throw new Test262Error('Promise incorrectly fulfilled.');
}, v => {
  assert.sameValue(v, obj, "reject reason");

  iter.next().then(({ done, value }) => {
    assert.sameValue(done, true, 'the iterator is completed');
    assert.sameValue(value, undefined, 'value is undefined');
  }).then($DONE, $DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);
