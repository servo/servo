// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack
description: >
  Throws a TypeError if NewTarget is undefined.
info: |
  AsyncDisposableStack ( )

  1. If NewTarget is undefined, throw a TypeError exception.
  ...
features: [explicit-resource-management]
---*/

assert.throws(TypeError, function() {
  AsyncDisposableStack();
});
