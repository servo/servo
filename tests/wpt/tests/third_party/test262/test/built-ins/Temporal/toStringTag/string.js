// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal-@@tostringtag
description: The @@toStringTag property of Temporal produces the correct value in toString
features: [Symbol.toStringTag, Temporal]
---*/

assert.sameValue(String(Temporal), "[object Temporal]");
