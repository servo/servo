// Copyright (C) 2016 The V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.isview
description: >
  Return true if arg is an instance from a subclass of DataView
info: |
  24.1.3.1 ArrayBuffer.isView ( arg )

  1. If Type(arg) is not Object, return false.
  2. If arg has a [[ViewedArrayBuffer]] internal slot, return true.
  3. Return false.
features: [class, DataView]
---*/

class DV extends DataView {}

var sample = new DV(new ArrayBuffer(1), 0, 0);

assert(ArrayBuffer.isView(sample));
