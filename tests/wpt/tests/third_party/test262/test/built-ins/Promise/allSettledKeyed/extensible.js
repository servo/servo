// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  Promise.allSettledKeyed is initially extensible.
info: |
  Unless specified otherwise, the [[Extensible]] internal slot
  of a built-in object initially has the value true.
features: [await-dictionary]
---*/

assert(Object.isExtensible(Promise.allSettledKeyed));
