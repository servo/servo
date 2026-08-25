// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Behaviour when property bag forms a date out of bounds of the current era
features: [Temporal, Intl.Era-monthcode]
---*/

// Last day of Showa era
const instance = new Temporal.PlainDate(1989, 1, 7, "japanese");

const result1 = instance.with({ day: 10 });
assert.notSameValue(result1.era, instance.era, "resulting day should have crossed an era boundary");

const result2 = instance.with({ month: 2 });
assert.notSameValue(result2.era, instance.era, "resulting month should have crossed an era boundary");
