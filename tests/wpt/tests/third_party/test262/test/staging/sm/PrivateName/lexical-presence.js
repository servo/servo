// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

// Verify that private fields are enabled.
class A {
  #x;
};

function assertThrowsSyntaxError(str) {
  assert.throws(SyntaxError, () => eval(str));       // Direct Eval
  assert.throws(SyntaxError, () => (1, eval)(str));  // Indirect Eval
  assert.throws(SyntaxError, () => Function(str));   // Function
}

assertThrowsSyntaxError(`
    class B {
        g() {
            return this.#x;
        }
    };
`);

assertThrowsSyntaxError(`
with (new class A { #x; }) {
    class B {
      #y;
      constructor() { return this.#x; }
    }
  }
`);

// Make sure we don't create a generic binding for #x.
with (new class {
  #x = 12;
}) {
  assert.sameValue('#x' in this, false);
}

// Try to fetch a non-existing field using different
// compilation contexts
function assertNonExisting(fetchCode) {
  source = `var X = class {
    b() {
      ${fetchCode}
    }
  }
  var a = new X;
  a.b()`
  assert.throws(SyntaxError, () => eval(source));
}

assertNonExisting(`return eval("this.#x")"`);
assertNonExisting(`return (1,eval)("this.#x")`);
assertNonExisting(`var f = Function("this.#x"); return f();`);

