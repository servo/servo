// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-getiter-sync-returns-string-throw.case
// - src/async-generators/default/async-class-expr-private-method.template
/*---
description: Non object returned by [Symbol.iterator]() - string (Async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorPrivateMethod
features: [Symbol.iterator, async-iteration, class-methods-private]
flags: [generated, async]
info: |
    ClassElement :
      PrivateMethodDefinition

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
    return '';
  }
};



var callCount = 0;

var C = class {
    async *#gen() {
        callCount += 1;
        yield* obj;
          throw new Test262Error('abrupt completion closes iter');

    }
    get gen() { return this.#gen; }
}

const c = new C();

// Test the private fields do not appear as properties before set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "#gen does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "#gen does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "#gen does not appear as an own property on C instance"
);

var iter = c.gen();

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

// Test the private fields do not appear as properties after set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "#gen does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "#gen does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "#gen does not appear as an own property on C instance"
);
