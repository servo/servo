// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Iterator.concat is not constructible.

  Built-in function objects that are not identified as constructors do not
  implement the [[Construct]] internal method unless otherwise specified in the
  description of a particular function.
features: [iterator-sequencing]
---*/

assert.throws(TypeError, () => {
  new Iterator.concat();
});
