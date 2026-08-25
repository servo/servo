// This file was procedurally generated from the following sources:
// - src/generators/yield-spread-arr-multiple.case
// - src/generators/default/class-decl-static-private-method.template
/*---
description: Use yield value in a array spread position (Static generator private method as a ClassDeclaration element)
esid: prod-GeneratorPrivateMethod
features: [generators, class-static-methods-private]
flags: [generated]
includes: [compareArray.js]
info: |
    ClassElement :
      static PrivateMethodDefinition

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

class C {
    static *#gen() {
        callCount += 1;
        yield [...yield yield];
    }
    static get gen() { return this.#gen; }
}

// Test the private fields do not appear as properties before set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "Private field '#gen' does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "Private field '#gen' does not appear as an own property on C constructor"
);

var iter = C.gen();

iter.next(false);
item = iter.next(['a', 'b', 'c']);
item = iter.next(item.value);

assert.compareArray(item.value, arr);
assert.sameValue(item.done, false);

assert.sameValue(callCount, 1);

// Test the private fields do not appear as properties before set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "Private field '#gen' does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "Private field '#gen' does not appear as an own property on C constructor"
);
