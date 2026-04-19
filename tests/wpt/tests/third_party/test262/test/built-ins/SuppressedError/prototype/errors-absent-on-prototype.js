// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-suppressederror-prototype-objects
description: >
  The SuppressedError prototype object isn't an SuppressedError instance.
info: |
  Properties of the SuppressedError Prototype Object

  The SuppressedError prototype object:
    ...
    - is not an Error instance or an SuppressedError instance and does not have an
      [[ErrorData]] internal slot.
    ...
features: [explicit-resource-management]
---*/

assert.sameValue(SuppressedError.prototype.hasOwnProperty("error"), false);
assert.sameValue(SuppressedError.prototype.hasOwnProperty("suppressed"), false);
