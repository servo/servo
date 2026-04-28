// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: >
    Intl.RelativeTimeFormat instance object extensibility
info: |
    17 ECMAScript Standard Built-in Objects:

    Unless specified otherwise, the [[Extensible]] internal slot
    of a built-in object initially has the value true.
features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(
  Object.isExtensible(new Intl.RelativeTimeFormat()),
  true,
  "Object.isExtensible(new Intl.RelativeTimeFormat()) returns true"
);
