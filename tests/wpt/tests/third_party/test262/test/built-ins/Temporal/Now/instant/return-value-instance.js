// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.now.instant
description: Temporal.Now.instant returns an Instant
features: [Temporal]
---*/

const instant = Temporal.Now.instant();
assert(instant instanceof Temporal.Instant);
