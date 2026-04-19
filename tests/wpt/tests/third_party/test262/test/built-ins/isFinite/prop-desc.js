// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isfinite-number
description: >
  Property descriptor for isFinite
includes: [propertyHelper.js]
---*/

verifyPrimordialCallableProperty(this, "isFinite", "isFinite", 1);
