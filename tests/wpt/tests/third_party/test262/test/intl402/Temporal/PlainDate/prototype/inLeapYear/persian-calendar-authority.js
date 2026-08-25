// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.inleapyear
description: >
  Leap years in the Persian calendar for 1206–1498 AP match the data published
  by the Iranian calendar authority
info: |
  The Iranian calendar authority has published the definitive leap year table
  for 1206–1498 AP.
  Source: https://calendar.ut.ac.ir/documents/2139738/7092644/Kabise+Shamsi+1206-1498.pdf
  Data: https://github.com/roozbehp/persiancalendar/blob/main/kabise.txt (CC0)
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "persian";

const leapYears = [
  1210, 1214, 1218, 1222, 1226, 1230, 1234, 1238, 1243, 1247,
  1251, 1255, 1259, 1263, 1267, 1271, 1276, 1280, 1284, 1288,
  1292, 1296, 1300, 1304, 1309, 1313, 1317, 1321, 1325, 1329,
  1333, 1337, 1342, 1346, 1350, 1354, 1358, 1362, 1366, 1370,
  1375, 1379, 1383, 1387, 1391, 1395, 1399, 1403, 1408, 1412,
  1416, 1420, 1424, 1428, 1432, 1436, 1441, 1445, 1449, 1453,
  1457, 1461, 1465, 1469, 1474, 1478, 1482, 1486, 1490, 1494, 1498
];

for (var year = 1206; year <= 1498; year++) {
  const date = Temporal.PlainDate.from({
    year,
    month: 1,
    day: 1,
    calendar
  });
  const expected = leapYears.includes(year);
  assert.sameValue(
    date.inLeapYear,
    expected,
    "Persian year " + year + " inLeapYear should be " + expected
  );
}
