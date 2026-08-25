// This file was procedurally generated from the following sources:
// - src/class-elements/private-method-referenced-from-static-method.case
// - src/class-elements/default/cls-decl.template
/*---
description: Private method referenced from a static method (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-methods-private, class]
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
  #f() { return 42; }
  static g() {
    return this.#f();
  }

}

assert.sameValue(C.g.call(new C()), 42);
assert.throws(TypeError, function() {
  C.g();
}, 'Accessed private method from an object which did not contain it');
