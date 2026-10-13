// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows is not constructible.

  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in
  the description of a particular function.
features: [iterator-chunking, generators]
---*/
function* g() {}
let iter = g();

assert.throws(TypeError, () => {
  new iter.windows(1);
});

assert.throws(TypeError, () => {
  new Iterator.prototype.windows(1);
});

assert.throws(TypeError, () => {
  new class extends Iterator {}.windows(1);
});
