// Copyright 2019 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: Checks basic handling of fractionalSecondDigits.
features: [Intl.DateTimeFormat-fractionalSecondDigits]
locale: [en]
---*/

const d1 = new Date(2019, 7, 10,  1, 2, 3, 234);
const d2 = new Date(2019, 7, 10,  1, 2, 3, 567);

let dtf = new Intl.DateTimeFormat(
    'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: undefined});
assert.sameValue(dtf.format(d1), "02:03", "no fractionalSecondDigits");
assert.sameValue(dtf.format(d2), "02:03", "no fractionalSecondDigits");

dtf = new Intl.DateTimeFormat(
    'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: 1});
assert.sameValue(dtf.format(d1), "02:03.2", "1 fractionalSecondDigits round down");
assert.sameValue(dtf.format(d2), "02:03.5", "1 fractionalSecondDigits round down");

dtf = new Intl.DateTimeFormat(
    'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: 2});
assert.sameValue(dtf.format(d1), "02:03.23", "2 fractionalSecondDigits round down");
assert.sameValue(dtf.format(d2), "02:03.56", "2 fractionalSecondDigits round down");

dtf = new Intl.DateTimeFormat(
    'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: 3});
assert.sameValue(dtf.format(d1), "02:03.234", "3 fractionalSecondDigits round down");
assert.sameValue(dtf.format(d2), "02:03.567", "3 fractionalSecondDigits round down");
