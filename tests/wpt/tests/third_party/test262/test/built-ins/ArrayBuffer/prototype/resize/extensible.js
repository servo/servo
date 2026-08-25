// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: ArrayBuffer.prototype.resize is extensible.
info: |
  ArrayBuffer.prototype.resize ( newLength )

  17 ECMAScript Standard Built-in Objects:
    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.
features: [resizable-arraybuffer]
---*/

assert(Object.isExtensible(ArrayBuffer.prototype.resize));
