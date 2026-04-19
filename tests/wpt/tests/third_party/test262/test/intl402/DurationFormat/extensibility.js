// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Intl.DurationFormat instance object extensibility
info: |
    17 ECMAScript Standard Built-in Objects:

    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.

features: [Intl.DurationFormat]
---*/

assert.sameValue(
  Object.isExtensible(new Intl.DurationFormat()),
  true,
  "Object.isExtensible(new Intl.DurationFormat()) returns true"
);


