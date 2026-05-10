// This file was procedurally generated from the following sources:
// - src/async-generators/yield-spread-arr-single.case
// - src/async-generators/default/async-class-decl-static-private-method.template
/*---
description: Use yield value in a array spread position (Static async generator method as a ClassDeclaration element)
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


    Array Initializer

    SpreadElement[Yield, Await]:
      ...AssignmentExpression[+In, ?Yield, ?Await]

---*/
var arr = ['a', 'b', 'c'];


var callCount = 0;

class C {
    static async *#gen() {
        callCount += 1;
        yield [...yield];
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

iter.next(false);
var item = iter.next(arr);

item.then(({ done, value }) => {
  assert.notSameValue(value, arr, 'value is a new array');
  assert(Array.isArray(value), 'value is an Array exotic object');
  assert.sameValue(value.length, 3)
  assert.sameValue(value[0], 'a');
  assert.sameValue(value[1], 'b');
  assert.sameValue(value[2], 'c');
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
