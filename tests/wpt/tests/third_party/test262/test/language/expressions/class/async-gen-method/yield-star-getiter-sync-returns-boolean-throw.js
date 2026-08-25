// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-getiter-sync-returns-boolean-throw.case
// - src/async-generators/default/async-class-expr-method.template
/*---
description: Non object returned by [Symbol.iterator]() - boolean (Async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorMethod
features: [Symbol.iterator, async-iteration]
flags: [generated, async]
info: |
    ClassElement :
      MethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


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
        i. Let syncMethod be ? GetMethod(obj, @@iterator).
        ii. Let syncIterator be ? Call(syncMethod, obj).
        iii. Return ? CreateAsyncFromSyncIterator(syncIterator).
    ...

    CreateAsyncFromSyncIterator(syncIterator)

    1. If Type(syncIterator) is not Object, throw a TypeError exception.
    ...

---*/
var obj = {
  [Symbol.iterator]() {
    return true;
  }
};



var callCount = 0;

var C = class { async *gen() {
    callCount += 1;
    yield* obj;
      throw new Test262Error('abrupt completion closes iter');

}}

var gen = C.prototype.gen;

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
