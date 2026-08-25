// This file was procedurally generated from the following sources:
// - src/generators/yield-spread-arr-multiple.case
// - src/generators/default/class-expr-private-method.template
/*---
description: Use yield value in a array spread position (Generator private method as a ClassExpression element)
esid: prod-GeneratorPrivateMethod
features: [generators, class-methods-private]
flags: [generated]
includes: [compareArray.js]
info: |
    ClassElement :
      PrivateMethodDefinition

    MethodDefinition :
      GeneratorMethod

    14.4 Generator Function Definitions

    GeneratorMethod :
      * PropertyName ( UniqueFormalParameters ) { GeneratorBody }


    Array Initializer

    SpreadElement[Yield, Await]:
      ...AssignmentExpression[+In, ?Yield, ?Await]

---*/
var arr = ['a', 'b', 'c'];
var item;

var callCount = 0;

var C = class {
    *#gen() {
        callCount += 1;
        yield [...yield yield];
    }
    get gen() { return this.#gen; }
}

const c = new C();

// Test the private fields do not appear as properties before set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "Private field '#gen' does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "Private field '#gen' does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "Private field '#gen' does not appear as an own property on C instance"
);

var iter = c.gen();

iter.next(false);
item = iter.next(['a', 'b', 'c']);
item = iter.next(item.value);

assert.compareArray(item.value, arr);
assert.sameValue(item.done, false);

assert.sameValue(callCount, 1);

// Test the private fields do not appear as properties after set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "Private field '#gen' does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "Private field '#gen' does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "Private field '#gen' does not appear as an own property on C instance"
);
