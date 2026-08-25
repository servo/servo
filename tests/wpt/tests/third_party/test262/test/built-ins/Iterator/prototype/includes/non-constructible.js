// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.includes
description: >
  Iterator.prototype.includes is not constructible.

  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in the
  description of a particular function.
features: [iterator-includes]
---*/

function* g() {
  yield 0;
}

let iter = g();

assert.throws(TypeError, function() {
  new iter.includes(0);
});

assert.throws(TypeError, function() {
  new iter.includes(0, 0);
});

assert.throws(TypeError, function() {
  new Iterator.prototype.includes(0);
});
