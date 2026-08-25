// This file was procedurally generated from the following sources:
// - src/class-elements/private-getter-is-not-a-own-property.case
// - src/class-elements/default/cls-decl.template
/*---
description: Private getter is not stored as an own property of objects (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-methods-private, class]
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

---*/


class C {
  get #m() { return "Test262"; }

  checkPrivateGetter() {
    assert.sameValue(this.hasOwnProperty("#m"), false);
    assert.sameValue("#m" in this, false);

    assert.sameValue(this.__lookupGetter__("#m"), undefined);

    assert.sameValue(this.#m, "Test262");

    return 0;
  }
}

let c = new C();
assert.sameValue(c.checkPrivateGetter(), 0);
