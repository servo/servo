// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Private setter is not clobbered by computed property
esid: sec-privatefieldget
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
features: [class-methods-private, class-fields-public, class]
---*/

class C {
  set #m(v) { this._v = v; }
  ["#m"] = 0;
  
  checkPrivateSetter() {
    assert.sameValue(this.hasOwnProperty("#m"), true);
    assert.sameValue("#m" in this, true);
  
    assert.sameValue(this["#m"], 0);
  
    this.#m = "Test262";
    assert.sameValue(this._v, "Test262");
  
    return 0;
  }
}

let c = new C();
assert.sameValue(c.checkPrivateSetter(), 0);
