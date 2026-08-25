// Copyright 2024 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/*---
description: Test public auto-accessors.
features: [decorators]
---*/

(function TestPublicAutoAccessor() {
  let name = 'x3';
  let symbol = Symbol();
  class C {
    accessor x0;
    accessor x1 = 1;
    accessor 'x2' = 2;
    accessor [name] = 3;
    accessor [symbol] = 4;
    accessor 1 = 5;
  }
  let c = new C();
  assert.sameValue(c.x0, undefined);
  assert.sameValue(c.x1, 1);
  assert.sameValue(c.x2, 2);
  assert.sameValue(c.x3, 3);
  assert.sameValue(c[symbol], 4);
  assert.sameValue(c[1], 5);
  c.x0 = 42;
  c.x1 = 43;
  c.x2 = 44;
  c.x3 = 45;
  c[symbol] = 46;
  c[1] = 47;
  assert.sameValue(c.x0, 42);
  assert.sameValue(c.x1, 43);
  assert.sameValue(c.x2, 44);
  assert.sameValue(c.x3, 45);
  assert.sameValue(c[symbol], 46);
  assert.sameValue(c[1], 47);
})();

(function TestRedeclaredPublicAutoaccessor() {
  let name = 'y';
  let symbol = Symbol();
  class C {
    accessor x = 0;
    accessor x = 1;
    accessor y = 2;
    accessor [name] = 3;
    accessor [symbol] = 4;
    accessor [symbol] = 5;
    accessor 1 = 6;
    accessor 1 = 7;
  }
  let c = new C();
  // Latest values should override previous ones.
  assert.sameValue(c.x, 1);
  assert.sameValue(c.y, 3);
  assert.sameValue(c[symbol], 5);
})();

(function TestDerivedPublicAutoAccessor() {
  let name = 'x3';
  let symbol = Symbol();
  let name2 = 'y3';
  let symbol2 = Symbol();
  class C {
    accessor x0;
    accessor x1 = 1;
    accessor 'x2' = 2;
    accessor [name] = 3;
    accessor [symbol] = 4;
    accessor 1 = 5;

    accessor y0 = 7;
    accessor y1 = 8;
    accessor 'y2' = 9;
    accessor [name2] = 10;
    accessor [symbol2] = 11;
    accessor 2 = 12;
  }
  class D extends C {
    accessor y0;
    accessor y1 = 13;
    accessor 'y2' = 14;
    accessor [name2] = 15;
    accessor [symbol2] = 16;
    accessor 2 = 17;
  }

  // Test inherited accessors.
  let d = new D();
  assert.sameValue(d.x0, undefined);
  assert.sameValue(d.x1, 1);
  assert.sameValue(d.x2, 2);
  assert.sameValue(d.x3, 3);
  assert.sameValue(d[symbol], 4);
  assert.sameValue(d[1], 5);

  // Test overridden accessors.
  assert.sameValue(d.y0, undefined);
  assert.sameValue(d.y1, 13);
  assert.sameValue(d.y2, 14);
  assert.sameValue(d.y3, 15);
  assert.sameValue(d[symbol2], 16);
  assert.sameValue(d[2], 17);
  d.y0 = 42;
  d.y1 = 43;
  d.y2 = 44;
  d.y3 = 45;
  d[symbol2] = 46;
  d[2] = 47;
  assert.sameValue(d.y0, 42);
  assert.sameValue(d.y1, 43);
  assert.sameValue(d.y2, 44);
  assert.sameValue(d.y3, 45);
  assert.sameValue(d[symbol2], 46);
  assert.sameValue(d[2], 47);

  // Test base class accessors.
  let c = new C();
  assert.sameValue(c.y0, 7);
  assert.sameValue(c.y1, 8);
  assert.sameValue(c.y2, 9);
  assert.sameValue(c.y3, 10);
  assert.sameValue(c[symbol2], 11);
  assert.sameValue(c[2], 12);
  c.y0 = 48;
  c.y1 = 49;
  c.y2 = 50;
  c.y3 = 51;
  c[symbol2] = 52;
  c[2] = 53;
  assert.sameValue(c.y0, 48);
  assert.sameValue(c.y1, 49);
  assert.sameValue(c.y2, 50);
  assert.sameValue(c.y3, 51);
  assert.sameValue(c[symbol2], 52);
  assert.sameValue(c[2], 53);
})();

(function TestAccessorsInPrototype() {
  const name = 'x2';
  class C {
    accessor x0 = 0;
    accessor x1 = 2;
    accessor [name] = 4;
    accessor 1 = 6;
  }

  const assertIsAccessorDescriptor = (key, value) => {
    let c = new C();
    assert.sameValue(
        Object.getOwnPropertyDescriptor(C.prototype, key).get.call(c), value);
    value++;
    Object.getOwnPropertyDescriptor(C.prototype, key).set.call(c, value);
    assert.sameValue(
        Object.getOwnPropertyDescriptor(C.prototype, key).get.call(c), value);
  };

  assertIsAccessorDescriptor('x0', 0);
  assertIsAccessorDescriptor('x1', 2);
  assertIsAccessorDescriptor('x2', 4);
  assertIsAccessorDescriptor(1, 6);
})();

(function TestPublicStaticAutoAccessor() {
  let name = 'x3';
  let symbol = Symbol();
  class C {
    static accessor x0;
    static accessor x1 = 1;
    static accessor 'x2' = 2;
    static accessor [name] = 3;
    static accessor [symbol] = 4;
    static accessor 1 = 5;
  }
  assert.sameValue(C.x0, undefined);
  assert.sameValue(C.x1, 1);
  assert.sameValue(C.x2, 2);
  assert.sameValue(C.x3, 3);
  assert.sameValue(C[symbol], 4);
  assert.sameValue(C[1], 5);
  C.x0 = 42;
  C.x1 = 43;
  C.x2 = 44;
  C.x3 = 45;
  C[symbol] = 46;
  C[1] = 47;
  assert.sameValue(C.x0, 42);
  assert.sameValue(C.x1, 43);
  assert.sameValue(C.x2, 44);
  assert.sameValue(C.x3, 45);
  assert.sameValue(C[symbol], 46);
  assert.sameValue(C[1], 47);
})();

(function TestRedeclaredPublicAutoaccessor() {
  let name = 'y';
  let symbol = Symbol();
  class C {
    static accessor x = 0;
    static accessor x = 1;
    static accessor y = 2;
    static accessor [name] = 3;
    static accessor [symbol] = 4;
    static accessor [symbol] = 5;
    static accessor 1 = 6;
    static accessor 1 = 7;
  }
  // Latest values should override previous ones.
  assert.sameValue(C.x, 1);
  assert.sameValue(C.y, 3);
  assert.sameValue(C[symbol], 5);
  assert.sameValue(C[1], 7);
})();

(function TestDerivedPublicStaticAutoAccessor() {
  let name = 'x3';
  let symbol = Symbol();
  let name2 = 'y3';
  let symbol2 = Symbol();
  class C {
    static accessor x0;
    static accessor x1 = 1;
    static accessor 'x2' = 2;
    static accessor [name] = 3;
    static accessor [symbol] = 4;
    static accessor 1 = 5;

    static accessor y0 = 6;
    static accessor y1 = 7;
    static accessor 'y2' = 8;
    static accessor [name2] = 9;
    static accessor [symbol2] = 10;
    static accessor 2 = 11;
  }
  class D extends C {
    static accessor y0;
    static accessor y1 = 12;
    static accessor 'y2' = 13;
    static accessor [name2] = 14;
    static accessor [symbol2] = 15;
    static accessor 2 = 16;
  }
  let assertThrowsTypeError = (code_string) => {
    assert.throws(TypeError, () => {eval(code_string)});
  };
  // Calling the accessor on the derived class passes such class as the receiver
  // to the generated getter/setter, which doesn't contain the static private
  // "accessor storage" variable and hence throws a TypeError.
  assertThrowsTypeError('D.x0');
  assertThrowsTypeError('D.x1');
  assertThrowsTypeError('D.x2');
  assertThrowsTypeError('D.x3');
  assertThrowsTypeError('D[symbol]');
  assertThrowsTypeError('D.x0 = 0');
  assertThrowsTypeError('D.x1 = 0');
  assertThrowsTypeError('D.x2 = 0');
  assertThrowsTypeError('D.x3 = 0');
  assertThrowsTypeError('D[symbol] = 0');
  assertThrowsTypeError('D[1]');

  // Test overridden accessors.
  assert.sameValue(D.y0, undefined);
  assert.sameValue(D.y1, 12);
  assert.sameValue(D.y2, 13);
  assert.sameValue(D.y3, 14);
  assert.sameValue(D[symbol2], 15);
  assert.sameValue(D[2], 16);

  D.y0 = 42;
  D.y1 = 43;
  D.y2 = 44;
  D.y3 = 45;
  D[symbol2] = 46;
  D[2] = 47;
  assert.sameValue(D.y0, 42);
  assert.sameValue(D.y1, 43);
  assert.sameValue(D.y2, 44);
  assert.sameValue(D.y3, 45);
  assert.sameValue(D[symbol2], 46);
  assert.sameValue(D[2], 47);

  // Test base class accessors.
  assert.sameValue(C.y0, 6);
  assert.sameValue(C.y1, 7);
  assert.sameValue(C.y2, 8);
  assert.sameValue(C.y3, 9);
  assert.sameValue(C[symbol2], 10);
  assert.sameValue(C[2], 11);
  C.y0 = 48;
  C.y1 = 49;
  C.y2 = 50;
  C.y3 = 51;
  C[symbol2] = 52;
  C[2] = 53;
  assert.sameValue(C.y0, 48);
  assert.sameValue(C.y1, 49);
  assert.sameValue(C.y2, 50);
  assert.sameValue(C.y3, 51);
  assert.sameValue(C[symbol2], 52);
  assert.sameValue(C[2], 53);
})();

(function TestStaticAccessorsInPrototype() {
  const name = 'x2';
  class C {
    static accessor x0 = 0;
    static accessor x1 = 2;
    static accessor [name] = 4;
    static accessor 1 = 6;
  }

  const assertIsAccessorDescriptor = (key, value) => {
    assert.sameValue(
        Object.getOwnPropertyDescriptor(C.prototype.constructor, key)
            .get.call(C),
        value);
    value++;
    Object.getOwnPropertyDescriptor(C.prototype.constructor, key)
        .set.call(C, value);
    assert.sameValue(
        Object.getOwnPropertyDescriptor(C.prototype.constructor, key)
            .get.call(C),
        value);
  };

  assertIsAccessorDescriptor('x0', 0);
  assertIsAccessorDescriptor('x1', 2);
  assertIsAccessorDescriptor('x2', 4);
  assertIsAccessorDescriptor(1, 6);
})();

(function TestPublicAutoAccessorLiteralNameOverrides() {
// Auto-accessors are override by getter/setter.
class A {
  #x = 1;
  accessor x = 2;
  get x () {return this.#x};
  set x (v) {this.#x = v};
  xValue() {return this.#x};
}
const a = new A();
assert.sameValue(a.x, 1); // Read from #x.
a.x = 3; // Set #x to 3.
assert.sameValue(a.x, 3); // Read from #x.
assert.sameValue(a.xValue(), 3); // Read from #x.

// Auto-accessor getter overrides user-defined getter.
// User-defined setter overrides auto-accessor setter.
class B {
  #x = 1;
  get x () {return this.#x};
  accessor x = 2;
  set x (v) {this.#x = v};
  xValue() {return this.#x};
}
const b = new B();
assert.sameValue(b.x, 2); // Read from accessor backing storage.
b.x = 3; // Set #x to 3.
assert.sameValue(b.x, 2); // Read from accessor backing storage.
assert.sameValue(b.xValue(), 3); // Read from #x.

// Auto-accessor overrdes user-defined getter and setter.
class C {
  #x = 1;
  get x () {return this.#x};
  set x (v) {this.#x = v};
  accessor x = 2;
  xValue() {return this.#x};
}
const c = new C(); // Read from accessor backing storage.
assert.sameValue(c.x, 2);
c.x = 3; // Set accessor backing storage.
assert.sameValue(c.x, 3); // Read from accessor backing storage.
assert.sameValue(c.xValue(), 1); // Read from #x.
})();

(function TestStaticPublicAutoAccessorLiteralNameOverrides() {
  // Auto-accessors are override by getter/setter.
  class A {
    static #x = 1;
    static accessor x = 2;
    static get x () {return this.#x};
    static set x (v) {this.#x = v};
    static xValue() {return this.#x};
  }
  assert.sameValue(A.x, 1);
  A.x = 3;
  assert.sameValue(A.x, 3);
  assert.sameValue(A.xValue(), 3);

  // Auto-accessor getter overrides user-defined getter.
  // User-defined setter overrides auto-accessor setter.
  class B {
    static #x = 1;
    static get x () {return this.#x};
    static accessor x = 2;
    static set x (v) {this.#x = v};
    static xValue() {return this.#x};
  }
  assert.sameValue(B.x, 2);
  B.x = 3; // Set #x to 3.
  assert.sameValue(B.x, 2); // Read from accessor backing storage.
  assert.sameValue(B.xValue(),3); // Read from #x.

  // Auto-accessor overrdes user-defined getter and setter.
  class C {
    static #x = 1;
    static get x () {return this.#x};
    static set x (v) {this.#x = v};
    static accessor x = 2;
    static xValue() {return this.#x};
  }
  assert.sameValue(C.x, 2);
  C.x = 3; // Set accessor backing storage.
  assert.sameValue(C.x, 3); // Read from accessor backing storage.
  assert.sameValue(C.xValue(), 1); // Read from #x.
})();

(function TestPublicAutoAccessorComputedNameOverrides() {
  let name = 'x';
  // Auto-accessors are override by getter/setter.
  class A {
    #x = 1;
    accessor [name] = 2;
    get x () {return this.#x};
    set x (v) {this.#x = v};
    xValue() {return this.#x};
  }
  const a = new A();
  assert.sameValue(a.x, 1); // Read from #x.
  a.x = 3; // Set #x to 3.
  assert.sameValue(a.x, 3); // Read from #x.
  assert.sameValue(a.xValue(), 3); // Read from #x.

  // Auto-accessor getter overrides user-defined getter.
  // User-defined setter overrides auto-accessor setter.
  class B {
    #x = 1;
    get x () {return this.#x};
    accessor [name] = 2;
    set x (v) {this.#x = v};
    xValue() {return this.#x};
  }
  const b = new B();
  assert.sameValue(b.x, 2); // Read from accessor backing storage.
  b.x = 3; // Set #x to 3.
  assert.sameValue(b.x, 2); // Read from accessor backing storage.
  assert.sameValue(b.xValue(), 3); // Read from #x.

  // Auto-accessor overrdes user-defined getter and setter.
  class C {
    #x = 1;
    get x () {return this.#x};
    set x (v) {this.#x = v};
    accessor [name] = 2;
    xValue() {return this.#x};
  }
  const c = new C(); // Read from accessor backing storage.
  assert.sameValue(c.x, 2);
  c.x = 3; // Set accessor backing storage.
  assert.sameValue(c.x, 3); // Read from accessor backing storage.
  assert.sameValue(c.xValue(), 1); // Read from #x.
})();

(function TestStaticPublicAutoAccessorComputedNameOverrides() {
  let name = 'x';
  // Auto-accessors are override by getter/setter.
  class A {
    static #x = 1;
    static accessor [name] = 2;
    static get x () {return this.#x};
    static set x (v) {this.#x = v};
    static xValue() {return this.#x};
  }
  assert.sameValue(A.x, 1);
  A.x = 3;
  assert.sameValue(A.x, 3);
  assert.sameValue(A.xValue(), 3);

  // Auto-accessor getter overrides user-defined getter.
  // User-defined setter overrides auto-accessor setter.
  class B {
    static #x = 1;
    static get x () {return this.#x};
    static accessor [name] = 2;
    static set x (v) {this.#x = v};
    static xValue() {return this.#x};
  }
  assert.sameValue(B.x, 2);
  B.x = 3; // Set #x to 3.
  assert.sameValue(B.x, 2); // Read from accessor backing storage.
  assert.sameValue(B.xValue(), 3); // Read from #x.

  // Auto-accessor overrdes user-defined getter and setter.
  class C {
    static #x = 1;
    static get x () {return this.#x};
    static set x (v) {this.#x = v};
    static accessor [name] = 2;
    static xValue() {return this.#x};
  }
  assert.sameValue(C.x, 2);
  C.x = 3; // Set accessor backing storage.
  assert.sameValue(C.x, 3); // Read from accessor backing storage.
  assert.sameValue(C.xValue(), 1); // Read from #x.
})();
