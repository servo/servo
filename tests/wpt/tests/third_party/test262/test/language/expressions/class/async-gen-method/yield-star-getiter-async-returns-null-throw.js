// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-getiter-async-returns-null-throw.case
// - src/async-generators/default/async-class-expr-method.template
/*---
description: Non object returned by [Symbol.asyncIterator]() - null (Async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorMethod
features: [Symbol.iterator, Symbol.asyncIterator, async-iteration]
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
    ...
    6. Let iterator be ? Call(method, obj).
    7. If Type(iterator) is not Object, throw a TypeError exception.
    ...

---*/
var obj = {
  get [Symbol.iterator]() {
    throw new Test262Error('it should not get Symbol.iterator');
  },
  [Symbol.asyncIterator]() {
    return null;
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
