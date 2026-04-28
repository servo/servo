// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  If message is undefined, no property will be set to the new instance
info: |
  AggregateError ( errors, message )

  ...
  5. If message is not undefined, then
    a. Let msg be ? ToString(message).
    b. Perform ! CreateMethodProperty(O, "message", msg).
  6. Return O.
features: [AggregateError]
---*/

var case1 = new AggregateError([], undefined);

assert.sameValue(
  Object.prototype.hasOwnProperty.call(case1, 'message'),
  false,
  'explicit'
);

var case2 = new AggregateError([]);

assert.sameValue(
  Object.prototype.hasOwnProperty.call(case2, 'message'),
  false,
  'implicit'
);
