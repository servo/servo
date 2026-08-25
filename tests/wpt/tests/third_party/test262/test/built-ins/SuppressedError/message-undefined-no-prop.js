// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-suppressederror-constructor
description: >
  If message is undefined, no property will be set to the new instance
info: |
  SuppressedError ( error, suppressed, message )

  ...
  5. If message is not undefined, then
    a. Let msg be ? ToString(message).
    b. Perform ! CreateMethodProperty(O, "message", msg).
  6. Return O.
features: [explicit-resource-management]
---*/

var case1 = new SuppressedError(undefined, undefined, undefined);

assert.sameValue(
  Object.prototype.hasOwnProperty.call(case1, 'message'),
  false,
  'explicit'
);

var case2 = new SuppressedError([]);

assert.sameValue(
  Object.prototype.hasOwnProperty.call(case2, 'message'),
  false,
  'implicit'
);
