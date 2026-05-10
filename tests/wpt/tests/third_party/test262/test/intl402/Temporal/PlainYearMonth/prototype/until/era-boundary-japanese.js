// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

const bce1 = Temporal.PlainYearMonth.from({ era: "bce", eraYear: 1, monthCode: "M06", calendar }, options);
const ce1 = Temporal.PlainYearMonth.from({ era: "ce", eraYear: 1, monthCode: "M06", calendar }, options);
const ce1872 = Temporal.PlainYearMonth.from({ era: "ce", eraYear: 1872, monthCode: "M12", calendar }, options);
const meiji6 = Temporal.PlainYearMonth.from({ era: "meiji", eraYear: 6, monthCode: "M01", calendar}, options);
const meiji7 = Temporal.PlainYearMonth.from({ era: "meiji", eraYear: 7, monthCode: "M01", calendar }, options);
const meiji45 = Temporal.PlainYearMonth.from({ era: "meiji", eraYear: 45, monthCode: "M05", calendar }, options);
const taisho1 = Temporal.PlainYearMonth.from({ era: "taisho", eraYear: 1, monthCode: "M08", calendar }, options);
const taisho6 = Temporal.PlainYearMonth.from({ era: "taisho", eraYear: 6, monthCode: "M03", calendar }, options);
const showa55 = Temporal.PlainYearMonth.from({ era: "showa", eraYear: 55, monthCode: "M03", calendar }, options);
const showa64 = Temporal.PlainYearMonth.from({ era: "showa", eraYear: 64, monthCode: "M01", calendar }, options);
const heisei1 = Temporal.PlainYearMonth.from({ era: "heisei", eraYear: 1, monthCode: "M02", calendar }, options);
const heisei30 = Temporal.PlainYearMonth.from({ era: "heisei", eraYear: 30, monthCode: "M03", calendar }, options);
const heisei31 = Temporal.PlainYearMonth.from({ era: "heisei", eraYear: 31, monthCode: "M04", calendar }, options);
const reiwa1 = Temporal.PlainYearMonth.from({ era: "reiwa", eraYear: 1, monthCode: "M06", calendar }, options);
const reiwa2 = Temporal.PlainYearMonth.from({ era: "reiwa", eraYear: 2, monthCode: "M03", calendar }, options);

const tests = [
  // From Heisei 30 (2018) to Reiwa 2 (2020) - crossing era boundary
  [
    heisei30, reiwa2,
    [2, 0, "2y from Heisei 30 March to Reiwa 2 March"],
    [0, 24, "24mo from Heisei 30 March to Reiwa 2 March"],
  ],
  [
    reiwa2, heisei30,
    [-2, 0, "-2y backwards from Heisei 30 March to Reiwa 2 March"],
    [0, -24, "-24mo backwards from Heisei 30 March to Reiwa 2 March"],
  ],
  // Within same year but different eras
  [
    heisei31, reiwa1,
    [0, 2, "2mo from Heisei 31 April to Reiwa 1 June"],
    [0, 2, "2mo from Heisei 31 April to Reiwa 1 June"],
  ],
  [
    reiwa1, heisei31,
    [0, -2, "-2mo backwards from Heisei 31 April to Reiwa 1 June"],
    [0, -2, "-2mo backwards from Heisei 31 April to Reiwa 1 June"],
  ],
  // From Showa 55 (1980) to Heisei 30 (2018) - crossing era boundary
  [
    showa55, heisei30,
    [38, 0, "38y from Showa 55 March to Heisei 30 March"],
    [0, 456, "456mo from Showa 55 March to Heisei 30 March"],
  ],
  [
    heisei30, showa55,
    [-38, 0, "-38y backwards from Showa 55 March to Heisei 30 March"],
    [0, -456, "-456mo backwards from Showa 55 March to Heisei 30 March"],
  ],
  // Within same year but different eras
  [
    showa64, heisei1,
    [0, 1, "1mo from Showa 64 January to Heisei 1 February"],
    [0, 1, "1mo from Showa 64 January to Heisei 1 February"],
  ],
  [
    heisei1, showa64,
    [0, -1, "-1mo from Showa 64 January to Heisei 1 February"],
    [0, -1, "-1mo from Showa 64 January to Heisei 1 February"],
  ],
  // From Taisho 6 (1917) to Showa 55 (1980) - crossing era boundary
  [
    taisho6, showa55,
    [63, 0, "63y from Taisho 6 March to Showa 55 March"],
    [0, 756, "756mo from Taisho 6 March to Showa 55 March"],
  ],
  [
    showa55, taisho6,
    [-63, 0, "-63y backwards from Taisho 6 March to Showa 55 March"],
    [0, -756, "-756mo backwards from Taisho 6 March to Showa 55 March"],
  ],
  // From Meiji 7 (1874) to Taisho 6 (1917) - crossing era boundary
  [
    meiji7, taisho6,
    [43, 2, "43y 2mo from Meiji 7 January to Taisho 6 March"],
    [0, 518, "518mo from Meiji 7 January to Taisho 6 March"],
  ],
  [
    taisho6, meiji7,
    [-43, -2, "-43y -2mo backwards from Meiji 7 January to Taisho 6 March"],
    [0, -518, "-518mo backwards from Meiji 7 January to Taisho 6 March"],
  ],
  // Within same year but different eras
  [
    meiji45, taisho1,
    [0, 3, "3mo from Meiji 45 May to Taisho 1 August"],
    [0, 3, "3mo from Meiji 45 May to Taisho 1 August"],
  ],
  [
    taisho1, meiji45,
    [0, -3, "-3mo backwards from Meiji 45 May to Taisho 1 August"],
    [0, -3, "-3mo backwards from Meiji 45 May to Taisho 1 August"],
  ],
  // Last pre-solar-calendar CE month to first solar-calendar month of Meiji era
  [
    ce1872, meiji6,
    [0, 1, "from month before solar Meiji era to first month"],
    [0, 1, "from month before solar Meiji era to first month"],
  ],
  [
    meiji6, ce1872,
    [0, -1, "backwards from month before solar Meiji era to first month"],
    [0, -1, "backwards from month before solar Meiji era to first month"],
  ],
  // CE-BCE boundary
  [
    bce1, ce1,
    [1, 0, "1y from 1 BCE to 1 CE"],
    [0, 12, "12mo from 1 BCE to 1 CE"],
  ],
  [
    ce1, bce1,
    [-1, 0, "-1y backwards from 1 BCE to 1 CE"],
    [0, -12, "-12mo backwards from 1 BCE to 1 CE"],
  ],
];

for (const [one, two, yearsTest, monthsTest] of tests) {
  let [years, months, descr] = yearsTest;
  let result = one.until(two, { largestUnit: "years" });
  TemporalHelpers.assertDuration(result, years, months, 0, 0, 0, 0, 0, 0, 0, 0, descr);

  [years, months, descr] = monthsTest;
  result = one.until(two, { largestUnit: "months" });
  TemporalHelpers.assertDuration(result, years, months, 0, 0, 0, 0, 0, 0, 0, 0, descr);
}
