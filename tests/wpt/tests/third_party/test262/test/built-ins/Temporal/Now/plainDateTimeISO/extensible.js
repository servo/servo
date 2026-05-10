// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.now.plaindatetimeiso
description: Temporal.Now.plainDateTimeISO is extensible.
features: [Temporal]
---*/

assert(
  Object.isExtensible(Temporal.Now.plainDateTimeISO),
  'Object.isExtensible(Temporal.Now.plainDateTimeISO) must return true'
);
