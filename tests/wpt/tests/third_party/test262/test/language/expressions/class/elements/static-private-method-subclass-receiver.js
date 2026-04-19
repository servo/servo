// This file was procedurally generated from the following sources:
// - src/class-elements/static-private-method-subclass-receiver.case
// - src/class-elements/default/cls-expr.template
/*---
description: Static private methods on the super-class cannot be called with sub-class as the receiver (field definitions in a class expression)
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
  static f() { return this.#g(); }
  static #g() { return 42; }

}

class D extends C {}
assert.sameValue(C.f(), 42);
assert.throws(TypeError, function() {
    D.f();
}, 'D does not contain static private method #g');
