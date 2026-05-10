// This file was procedurally generated from the following sources:
// - src/async-generators/yield-spread-arr-multiple.case
// - src/async-generators/default/async-class-expr-private-method.template
/*---
description: Use yield value in a array spread position (Async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorPrivateMethod
features: [async-iteration, class-methods-private]
flags: [generated, async]
includes: [compareArray.js]
info: |
    ClassElement :
      PrivateMethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    Array Initializer

    SpreadElement[Yield, Await]:
      ...AssignmentExpression[+In, ?Yield, ?Await]

---*/
var arr = ['a', 'b', 'c'];
var item;


var callCount = 0;

var C = class {
    async *#gen() {
        callCount += 1;
        yield [...yield yield];
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

iter.next(false);
item = iter.next(['a', 'b', 'c']);

item.then(({ done, value }) => {
  item = iter.next(value);

  item.then(({ done, value }) => {
    assert.compareArray(value, arr);
    assert.sameValue(done, false);
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
