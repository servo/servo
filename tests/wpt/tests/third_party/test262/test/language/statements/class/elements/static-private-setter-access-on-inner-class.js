// This file was procedurally generated from the following sources:
// - src/class-elements/static-private-setter-access-on-inner-class.case
// - src/class-elements/default/cls-decl.template
/*---
description: static private setter access inside of an inner class (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-static-methods-private, class-static-fields-public, class]
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
  static set #f(v) {
    return this._v = v;
  }

  static Inner = class {
    static access(o) {
      o.#f = 'Test262';
    }
  }

}

C.Inner.access(C)
assert.sameValue(C._v, 'Test262');
assert.throws(TypeError, function() {
  C.Inner.access(C.Inner);
}, 'Accessed static private setter from an object which did not contain it');
