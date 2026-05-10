// This file was procedurally generated from the following sources:
// - src/async-generators/yield-star-next-then-non-callable-undefined-fulfillpromise.case
// - src/async-generators/default/async-class-expr-static-private-method.template
/*---
description: FulfillPromise if next().then is not-callable (undefined) (Static async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorPrivateMethod
features: [Symbol.iterator, Symbol.asyncIterator, async-iteration, class-static-methods-private]
flags: [generated, async]
info: |
    ClassElement :
      static PrivateMethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


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

var C = class {
    static async *#gen() {
        callCount += 1;
        yield* obj;
          throw new Test262Error('completion closes iter');

    }
    static get gen() { return this.#gen; }
}

// Test the private fields do not appear as properties before set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "#gen does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "#gen does not appear as an own property on C constructor"
);

var iter = C.gen();

iter.next().then(({ value, done }) => {
  assert.sameValue(value, 42);
  assert.sameValue(done, false);
}).then($DONE, $DONE);

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
