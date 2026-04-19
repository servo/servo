// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Date difference works correctly across era boundaries
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

const bce1 = Temporal.PlainDate.from({ era: "bce", eraYear: 1, monthCode: "M06", day: 1, calendar }, options);
const ce1 = Temporal.PlainDate.from({ era: "ce", eraYear: 1, monthCode: "M06", day: 1, calendar }, options);
const ce1872 = Temporal.PlainDate.from({ era: "ce", eraYear: 1872, monthCode: "M12", day: 31, calendar }, options);
const meiji6 = Temporal.PlainDate.from({ era: "meiji", eraYear: 6, monthCode: "M01", day: 1, calendar}, options);
const meiji7 = Temporal.PlainDate.from({ era: "meiji", eraYear: 7, monthCode: "M01", day: 15, calendar }, options);
const meiji45 = Temporal.PlainDate.from({ era: "meiji", eraYear: 45, monthCode: "M05", day: 19, calendar }, options);
const taisho1 = Temporal.PlainDate.from({ era: "taisho", eraYear: 1, monthCode: "M08", day: 9, calendar }, options);
const taisho6 = Temporal.PlainDate.from({ era: "taisho", eraYear: 6, monthCode: "M03", day: 15, calendar }, options);
const taisho15 = Temporal.PlainDate.from({ era: "taisho", eraYear: 15, monthCode: "M05", day: 20, calendar }, options);
const showa1 = Temporal.PlainDate.from({ era: "showa", eraYear: 1, monthCode: "M12", day: 30, calendar }, options);
const showa55 = Temporal.PlainDate.from({ era: "showa", eraYear: 55, monthCode: "M03", day: 15, calendar }, options);
const showa64 = Temporal.PlainDate.from({ era: "showa", eraYear: 64, monthCode: "M01", day: 3, calendar }, options);
const heisei1 = Temporal.PlainDate.from({ era: "heisei", eraYear: 1, monthCode: "M02", day: 27, calendar }, options);
const heisei30 = Temporal.PlainDate.from({ era: "heisei", eraYear: 30, monthCode: "M03", day: 15, calendar }, options);
const heisei31 = Temporal.PlainDate.from({ era: "heisei", eraYear: 31, monthCode: "M04", day: 1, calendar }, options);
const reiwa1 = Temporal.PlainDate.from({ era: "reiwa", eraYear: 1, monthCode: "M06", day: 1, calendar }, options);
const reiwa2 = Temporal.PlainDate.from({ era: "reiwa", eraYear: 2, monthCode: "M03", day: 15, calendar }, options);

const tests = [
  // From Heisei 30 (2018) to Reiwa 2 (2020) - crossing era boundary
  [
    heisei30, reiwa2,
    [-2, 0, 0, 0, "-2y backwards from Heisei 30 March to Reiwa 2 March"],
    [0, -24, 0, 0, "-24mo backwards from Heisei 30 March to Reiwa 2 March"],
  ],
  [
    reiwa2, heisei30,
    [2, 0, 0, 0, "2y from Heisei 30 March to Reiwa 2 March"],
    [0, 24, 0, 0, "24mo from Heisei 30 March to Reiwa 2 March"],
  ],
  // Within same year but different eras
  [
    heisei31, reiwa1,
    [0, -2, 0, 0, "-2mo backwards from Heisei 31 April to Reiwa 1 June"],
    [0, -2, 0, 0, "-2mo backwards from Heisei 31 April to Reiwa 1 June"],
  ],
  [
    reiwa1, heisei31,
    [0, 2, 0, 0, "2mo from Heisei 31 April to Reiwa 1 June"],
    [0, 2, 0, 0, "2mo from Heisei 31 April to Reiwa 1 June"],
  ],
  // From Showa 55 (1980) to Heisei 30 (2018) - crossing era boundary
  [
    showa55, heisei30,
    [-38, 0, 0, 0, "-38y backwards from Showa 55 March to Heisei 30 March"],
    [0, -456, 0, 0, "-456mo backwards from Showa 55 March to Heisei 30 March"],
  ],
  [
    heisei30, showa55,
    [38, 0, 0, 0, "38y from Showa 55 March to Heisei 30 March"],
    [0, 456, 0, 0, "456mo from Showa 55 March to Heisei 30 March"],
  ],
  // Within same year but different eras
  [
    showa64, heisei1,
    [0, -1, 0, -24, "-1mo -24d from Showa 64 January 3 to Heisei 1 February 27"],
    [0, -1, 0, -24, "-1mo -24d from Showa 64 January 3 to Heisei 1 February 27"],
  ],
  [
    heisei1, showa64,
    [0, 1, 0, 24, "1mo 24d from Showa 64 January 3 to Heisei 1 February 27"],
    [0, 1, 0, 24, "1mo 24d from Showa 64 January 3 to Heisei 1 February 27"],
  ],
  // From Taisho 6 (1917) to Showa 55 (1980) - crossing era boundary
  [
    taisho6, showa55,
    [-63, 0, 0, 0, "-63y backwards from Taisho 6 March to Showa 55 March"],
    [0, -756, 0, 0, "-756mo backwards from Taisho 6 March to Showa 55 March"],
  ],
  [
    showa55, taisho6,
    [63, 0, 0, 0, "63y from Taisho 6 March to Showa 55 March"],
    [0, 756, 0, 0, "756mo from Taisho 6 March to Showa 55 March"],
  ],
  // Within same year but different eras
  [
    taisho15, showa1,
    [0, -7, 0, -10, "-7mo -10d backwards from Taisho 15 July 20 to Showa 1 December 30"],
    [0, -7, 0, -10, "-7mo -10d backwards from Taisho 15 July 20 to Showa 1 December 30"],
  ],
  [
    showa1, taisho15,
    [0, 7, 0, 10, "7mo 10d from Taisho 15 July 20 to Showa 1 December 30"],
    [0, 7, 0, 10, "7mo 10d from Taisho 15 July 20 to Showa 1 December 30"],
  ],
  // From Meiji 7 (1874) to Taisho 6 (1917) - crossing era boundary
  [
    meiji7, taisho6,
    [-43, -2, 0, 0, "-43y -2mo backwards from Meiji 7 January to Taisho 6 March"],
    [0, -518, 0, 0, "-518mo backwards from Meiji 7 January to Taisho 6 March"],
  ],
  [
    taisho6, meiji7,
    [43, 2, 0, 0, "43y 2mo from Meiji 7 January to Taisho 6 March"],
    [0, 518, 0, 0, "518mo from Meiji 7 January to Taisho 6 March"],
  ],
  // Within same year but different eras
  [
    meiji45, taisho1,
    [0, -2, 0, -21, "-2mo -21d backwards from Meiji 45 May 19 to Taisho 1 August 9"],
    [0, -2, 0, -21, "-2mo -21d backwards from Meiji 45 May 19 to Taisho 1 August 9"],
  ],
  [
    taisho1, meiji45,
    [0, 2, 0, 21, "2mo 21d from Meiji 45 May 19 to Taisho 1 August 9"],
    [0, 2, 0, 21, "2mo 21d from Meiji 45 May 19 to Taisho 1 August 9"],
  ],
  // Last pre-solar-calendar CE day to first solar-calendar day of Meiji era
  [
    ce1872, meiji6,
    [0, 0, 0, -1, "backwards from day before solar Meiji era to first day"],
    [0, 0, 0, -1, "backwards from day before solar Meiji era to first day"],
  ],
  [
    meiji6, ce1872,
    [0, 0, 0, 1, "from day before solar Meiji era to first day"],
    [0, 0, 0, 1, "from day before solar Meiji era to first day"],
  ],
  // CE-BCE boundary
  [
    bce1, ce1,
    [-1, 0, 0, 0, "-1y backwards from 1 BCE to 1 CE"],
    [0, -12, 0, 0, "-12mo backwards from 1 BCE to 1 CE"],
  ],
  [
    ce1, bce1,
    [1, 0, 0, 0, "1y from 1 BCE to 1 CE"],
    [0, 12, 0, 0, "12mo from 1 BCE to 1 CE"],
  ],
];

for (const [one, two, yearsTest, monthsTest] of tests) {
  let [years, months, weeks, days, descr] = yearsTest;
  let result = one.since(two, { largestUnit: "years" });
  TemporalHelpers.assertDuration(result, years, months, weeks, days, 0, 0, 0, 0, 0, 0, descr);

  [years, months, weeks, days, descr] = monthsTest;
  result = one.since(two, { largestUnit: "months" });
  TemporalHelpers.assertDuration(result, years, months, weeks, days, 0, 0, 0, 0, 0, 0, descr);

  const oneISO = one.withCalendar("iso8601");
  const twoISO = two.withCalendar("iso8601");

  const resultWeeks = one.since(two, { largestUnit: "weeks" });
  const resultWeeksISO = oneISO.since(twoISO, { largestUnit: "weeks" });
  TemporalHelpers.assertDurationsEqual(resultWeeks, resultWeeksISO,
    `${one.year}-${one.monthCode}-${one.day} : ${two.year}-${two.monthCode}-${two.day} largestUnit weeks`);

  const resultDays = one.since(two);
  const resultDaysISO = oneISO.since(twoISO);
  TemporalHelpers.assertDurationsEqual(resultDays, resultDaysISO,
    `${one.year}-${one.monthCode}-${one.day} : ${two.year}-${two.monthCode}-${two.day} largestUnit days`);
}
