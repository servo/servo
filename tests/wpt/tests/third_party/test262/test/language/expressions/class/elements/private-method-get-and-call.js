// This file was procedurally generated from the following sources:
// - src/class-elements/private-method-get-and-call.case
// - src/class-elements/default/cls-expr.template
/*---
description: Function returned by a private method can be called with other values as 'this' (field definitions in a class expression)
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


var C = class {
  #m() { return this._v; }
    
  getPrivateMethod() {
      return this.#m;
  }

}

let c = new C();

let o1 = {_v: 'test262'};
let o2 = {_v: 'foo'}; 
assert.sameValue(c.getPrivateMethod().call(o1), 'test262');
assert.sameValue(c.getPrivateMethod().call(o2), 'foo');
