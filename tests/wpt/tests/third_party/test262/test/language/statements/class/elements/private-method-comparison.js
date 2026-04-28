// This file was procedurally generated from the following sources:
// - src/class-elements/private-method-comparison.case
// - src/class-elements/default/cls-decl.template
/*---
description: PrivateFieldGet of a private method returns the same function object to every instance of the same class (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class, class-methods-private]
flags: [generated]
info: |
    PrivateFieldGet (P, O)
      1. Assert: P is a Private Name.
      2. If O is not an object, throw a TypeError exception.
      3. If P.[[Kind]] is "field",
        a. Let entry be PrivateFieldFind(P, O).
        b. If entry is empty, throw a TypeError exception.
        c. Return entry.[[PrivateFieldValue]].
      4. Perform ? PrivateBrandCheck(O, P).
      5. If P.[[Kind]] is "method",
        a. Return P.[[Value]].
      6. Else,
        a. Assert: P.[[Kind]] is "accessor".
        b. If P does not have a [[Get]] field, throw a TypeError exception.
        c. Let getter be P.[[Get]].
        d. Return ? Call(getter, O).

    PrivateBrandCheck(O, P)
      1. If O.[[PrivateBrands]] does not contain an entry e such that SameValue(e, P.[[Brand]]) is true,
        a. Throw a TypeError exception.

---*/


class C {
  #m() { return 'test262'; }
    
  getPrivateMethod() {
      return this.#m;
  }

}

let c1 = new C();
let c2 = new C();

assert.sameValue(c1.getPrivateMethod(), c2.getPrivateMethod());
