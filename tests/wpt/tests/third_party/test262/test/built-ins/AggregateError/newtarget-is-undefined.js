// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  NewTarget is undefined
info: |
  AggregateError ( errors, message )

  1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.

features: [AggregateError]
---*/

var obj = AggregateError([], '');

assert.sameValue(Object.getPrototypeOf(obj), AggregateError.prototype);
assert(obj instanceof AggregateError);
