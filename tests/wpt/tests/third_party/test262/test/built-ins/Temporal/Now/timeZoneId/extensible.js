// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.timezoneid
description: Temporal.Now.timeZoneId is extensible.
info: |
  ## 17 ECMAScript Standard Built-in Objects

  Unless specified otherwise, the [[Extensible]] internal slot
  of a built-in object initially has the value true.
features: [Temporal]
---*/

assert(
  Object.isExtensible(Temporal.Now.timeZoneId),
  'Object.isExtensible(Temporal.Now.timeZoneId) must return true'
);
