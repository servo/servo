// This file was procedurally generated from the following sources:
// - src/async-generators/yield-promise-reject-next-catch.case
// - src/async-generators/default/async-class-expr-static-private-method.template
/*---
description: yield Promise.reject(value) is treated as throw value (Static async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorPrivateMethod
features: [async-iteration, class-static-methods-private]
flags: [generated, async]
info: |
    ClassElement :
      static PrivateMethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }

---*/
let error = new Error();


var callCount = 0;

var C = class {
    static async *#gen() {
        callCount += 1;
        yield Promise.reject(error);
        yield "unreachable";
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

iter.next().then(() => {
  throw new Test262Error("Promise incorrectly resolved.");
}).catch(rejectValue => {
  // yield Promise.reject(error);
  assert.sameValue(rejectValue, error);

  iter.next().then(({done, value}) => {
    // iter is closed now.
    assert.sameValue(done, true, "The value of IteratorResult.done is `true`");
    assert.sameValue(value, undefined, "The value of IteratorResult.value is `undefined`");
  }).then($DONE, $DONE);
});

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
