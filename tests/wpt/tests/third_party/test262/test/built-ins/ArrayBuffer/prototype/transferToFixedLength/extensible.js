// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfertofixedlength
description: ArrayBuffer.prototype.transferToFixedLength is extensible.
info: |
  ArrayBuffer.prototype.transferToFixedLength ( [ newLength ] )

  17 ECMAScript Standard Built-in Objects:
    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.
features: [arraybuffer-transfer]
---*/

assert(Object.isExtensible(ArrayBuffer.prototype.transferToFixedLength));
