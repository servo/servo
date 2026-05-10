// This file was procedurally generated from the following sources:
// - src/class-elements/static-private-method-referenced-from-instance-method.case
// - src/class-elements/default/cls-decl.template
/*---
description: Static private method referenced from an instance method (field definitions in a class declaration)
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


class C {
  static #f() { return 42; }
  g() {
    return this.#f();
  }

}

assert.sameValue(new C().g.call(C), 42);
assert.throws(TypeError, function() {
  new C().g();
}, 'Accessed static private method from an object which did not contain it');
