// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Iterator.prototype.chunks is not constructible.

  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in
  the description of a particular function.
features: [iterator-chunking, generators, class]
---*/
function* g() {}
let iter = g();

assert.throws(TypeError, () => {
  new iter.chunks(1);
});

assert.throws(TypeError, () => {
  new Iterator.prototype.chunks(1);
});

assert.throws(TypeError, () => {
  new class extends Iterator {}.chunks(1);
});
