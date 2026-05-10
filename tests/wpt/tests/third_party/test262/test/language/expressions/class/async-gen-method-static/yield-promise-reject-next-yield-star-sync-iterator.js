// This file was procedurally generated from the following sources:
// - src/async-generators/yield-promise-reject-next-yield-star-sync-iterator.case
// - src/async-generators/default/async-class-expr-static-method.template
/*---
description: yield * (async iterator) is treated as throw value (Static async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorMethod
features: [async-iteration]
flags: [generated, async]
info: |
    ClassElement :
      static MethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }

---*/
let error = new Error();
let iterable = [
  Promise.reject(error),
  "unreachable"
];


var callCount = 0;

var C = class { static async *gen() {
    callCount += 1;
    yield * iterable;
}}

var gen = C.gen;

var iter = gen();

iter.next().then(() => {
  throw new Test262Error("Promise incorrectly resolved.");
}, rejectValue => {
  // yield Promise.reject(error);
  assert.sameValue(rejectValue, error);

  iter.next().then(({done, value}) => {
    // iter is closed now.
    assert.sameValue(done, true, "The value of IteratorResult.done is `true`");
    assert.sameValue(value, undefined, "The value of IteratorResult.value is `undefined`");
  }).then($DONE, $DONE);
}).catch($DONE);

assert.sameValue(callCount, 1);
