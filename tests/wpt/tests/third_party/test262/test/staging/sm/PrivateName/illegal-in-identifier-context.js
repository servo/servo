// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Private names can't appear in contexts where plain identifiers are expected.

// Private names as binding identifiers.
assert.throws(SyntaxError, () => eval(`var #a;`));
assert.throws(SyntaxError, () => eval(`let #a;`));
assert.throws(SyntaxError, () => eval(`const #a = 0;`));
assert.throws(SyntaxError, () => eval(`function #a(){}`));
assert.throws(SyntaxError, () => eval(`function f(#a){}`));

// With escape sequences (leading and non-leading case).
assert.throws(SyntaxError, () => eval(String.raw`var #\u0061;`));
assert.throws(SyntaxError, () => eval(String.raw`var #a\u0061;`));
assert.throws(SyntaxError, () => eval(String.raw`let #\u0061;`));
assert.throws(SyntaxError, () => eval(String.raw`let #a\u0061;`));
assert.throws(SyntaxError, () => eval(String.raw`const #\u0061 = 0;`));
assert.throws(SyntaxError, () => eval(String.raw`const #a\u0061 = 0;`));
assert.throws(SyntaxError, () => eval(String.raw`function #\u0061(){}`));
assert.throws(SyntaxError, () => eval(String.raw`function #a\u0061(){}`));
assert.throws(SyntaxError, () => eval(String.raw`function f(#\u0061){}`));
assert.throws(SyntaxError, () => eval(String.raw`function f(#a\u0061){}`));


// Private names as label identifiers.
assert.throws(SyntaxError, () => eval(`#a: ;`));

// With escape sequences (leading and non-leading case).
assert.throws(SyntaxError, () => eval(String.raw`#\u0061: ;`));
assert.throws(SyntaxError, () => eval(String.raw`#a\u0061: ;`));


// Private names as identifier references.
assert.throws(SyntaxError, () => eval(`#a = 0;`));
assert.throws(SyntaxError, () => eval(`typeof #a;`));

// With escape sequences (leading and non-leading case).
assert.throws(SyntaxError, () => eval(String.raw`#\u0061 = 0;`));
assert.throws(SyntaxError, () => eval(String.raw`#a\u0061 = 0;`));
assert.throws(SyntaxError, () => eval(String.raw`typeof #\u0061;`));
assert.throws(SyntaxError, () => eval(String.raw`typeof #a\u0061;`));


