// Copyright (C) 2025 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - Error.isError
description: |
  pending
esid: pending
---*/

// Test non-object input should return false
assert.sameValue(Error.isError(null), false);
assert.sameValue(Error.isError(undefined), false);
assert.sameValue(Error.isError(123), false);
assert.sameValue(Error.isError("string"), false);

// Test plain objects should return false
assert.sameValue(Error.isError({}), false);
assert.sameValue(Error.isError(new Object()), false);

// Test various error objects should return true
assert.sameValue(Error.isError(new Error()), true);
assert.sameValue(Error.isError(new TypeError()), true);
assert.sameValue(Error.isError(new RangeError()), true);

