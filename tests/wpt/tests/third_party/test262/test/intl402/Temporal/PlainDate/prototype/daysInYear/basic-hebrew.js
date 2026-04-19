// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.daysinyear
description: Days in year in the Hebrew calendar
info: |
  There are discrepancies in these data between ICU4C 77.1 and ICU4C 78.1,
  which will affect implementations relying on ICU4C.
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

const sampleData = {
  5730: 383,
  5731: 354,
  5732: 355,
  5733: 383,
  5734: 355,
  5735: 354,
  5736: 385,
  5737: 353,
  5738: 384,
  5739: 355,
  5740: 355,
  5741: 383,
  5742: 354,
  5743: 355,
  5744: 385,
  5745: 354,
  5746: 383,
  5747: 355,
  5748: 354,
  5749: 383,
  5750: 355,
  5751: 354,
  5752: 385,
  5753: 353,
  5754: 355,
  5755: 384,
  5756: 355,
  5757: 383,
  5758: 354,
  5759: 355,
  5760: 385,
  5761: 353,
  5762: 354,
  5763: 385,
  5764: 355,
  5765: 383,
  5766: 354,
  5767: 355,
  5768: 383,
  5769: 354,
  5770: 355,
  5771: 385,
  5772: 354,
  5773: 353,
  5774: 385,
  5775: 354,
  5776: 385,
  5777: 353,
  5778: 354,
  5779: 385,
  5780: 355,
  5781: 353,
  5782: 384,
  5783: 355,
  5784: 383,
  5785: 355,
  5786: 354,
  5787: 385,
  5788: 355,
  5789: 354,
  5790: 383,
  5791: 355,
  5792: 354,
  5793: 383,
  5794: 355,
  5795: 385,
  5796: 354,
  5797: 353,
  5798: 385,
  5799: 354,
  5800: 355,
  5801: 383,
  5802: 354,
  5803: 385,
  5804: 353,
  5805: 355,
  5806: 384,
  5807: 355,
  5808: 353,
  5809: 384,
}

for (var [year, days] of Object.entries(sampleData)) {
    const date = Temporal.PlainDate.from({
        year,
        month: 1,
        calendar, day: 1
    });
    assert.sameValue(date.daysInYear, days, `days in year ${year}`);
}
