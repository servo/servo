// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer.prototype.grow
description: SharedArrayBuffer.prototype.grow is extensible.
info: |
  SharedArrayBuffer.prototype.grow ( newLength )

  17 ECMAScript Standard Built-in Objects:
    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

assert(Object.isExtensible(SharedArrayBuffer.prototype.grow));
