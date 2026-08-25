// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-getiter-async-not-callable-object-throw.case
// - src/async-generators/default/async-class-expr-private-method.template
/*---
description: Throws a TypeError on a non-callable [Symbol.asyncIterator] (object) (Async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorPrivateMethod
features: [Symbol.iterator, Symbol.asyncIterator, async-iteration, class-methods-private]
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
    ...

    GetMethod ( V, P )

    ...
    2. Let func be ? GetV(V, P).
    3. If func is either undefined or null, return undefined.
    4. If IsCallable(func) is false, throw a TypeError exception.
    ...

---*/
var obj = {
  get [Symbol.iterator]() {
    throw new Test262Error('it should not get Symbol.iterator');
  },
  [Symbol.asyncIterator]: {}
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
