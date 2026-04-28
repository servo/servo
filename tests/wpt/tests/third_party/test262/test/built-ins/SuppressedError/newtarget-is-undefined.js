// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-suppressederror-constructor
description: >
  NewTarget is undefined
info: |
  SuppressedError ( error, suppressed, message )

  1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.

features: [explicit-resource-management]
---*/

var obj = SuppressedError();

assert.sameValue(Object.getPrototypeOf(obj), SuppressedError.prototype);
assert(obj instanceof SuppressedError);
