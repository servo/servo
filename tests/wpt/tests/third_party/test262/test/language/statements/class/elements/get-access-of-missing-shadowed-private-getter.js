// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Trying to get from PrivateName without [[Get]] throws TypeError
esid: sec-privatefieldget
info: |
  PrivateFieldGet (P, O )
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

class A {
  get #f() {
    throw new Test262Error();
  }
}

class C extends A {
  set #f(v) {
    throw new Test262Error();
  }

  getAccess() {
    return this.#f;
  }
}

let c = new C();
assert.throws(TypeError, function() {
  c.getAccess();
}, 'subclass private accessor should shadow super class private accessor');

class B {
  get #f() {
    throw new Test262Error();
  }

  Inner = class {
    set #f(v) {
      throw new Test262Error();
    }

    getAccess() {
      return this.#f;
    }
  }
}

let b = new B();
let innerB = new b.Inner();
assert.throws(TypeError, function() {
  innerB.getAccess();
}, 'inner class private accessor should shadow outer class private accessor');

class D {
  set #f(v) {
    throw new Test262Error();
  }

  Inner = class {
    get #f() {
      throw new Test262Error();
    }
  }

  getAccess() {
    return this.#f;
  }
}

let d = new D();
assert.throws(TypeError, function() {
  d.getAccess();
}, 'inner class private accessor should not be visible to outer class private accessor');

