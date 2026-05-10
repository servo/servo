// This file was procedurally generated from the following sources:
// - src/class-elements/static-private-method-access-on-inner-arrow-function.case
// - src/class-elements/default/cls-expr.template
/*---
description: Static private method access inside of an arrow function (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-static-methods-private, class]
flags: [generated]
info: |
    PrivateFieldGet (P, O)
      1. Assert: P is a Private Name.
      2. If O is not an object, throw a TypeError exception.
      3. If P.[[Kind]] is "field",
          ...
      4. Perform ? PrivateBrandCheck(O, P).
      5. If P.[[Kind]] is "method",
        a. Return P.[[Value]].
      ...

    PrivateBrandCheck(O, P)
      1. If O.[[PrivateBrands]] does not contain an entry e such that SameValue(e, P.[[Brand]]) is true,
        a. Throw a TypeError exception.

---*/


var C = class {
  static #f() { return 42; }
  static g() {
    const arrowFunction = () => {
      return this.#f();
    };

    return arrowFunction();
  }

}

assert.sameValue(C.g(), 42);
assert.throws(TypeError, function() {
  C.g.call({});
}, 'Accessed static private method from an object which did not contain it');
