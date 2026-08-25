// Copyright (c) 2014 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.


/*---
esid: sec-array.prototype.concat
description: Array.prototype.concat no prototype
---*/
assert.sameValue(
  Array.prototype.concat.prototype,
  void 0,
  'The value of Array.prototype.concat.prototype is expected to be void 0'
);
