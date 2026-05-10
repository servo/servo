// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  ArrayBuffer.prototype.slice is extensible.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  17 ECMAScript Standard Built-in Objects:
    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.
---*/

assert(Object.isExtensible(ArrayBuffer.prototype.slice));
