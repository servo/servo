// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.daysinyear
description: Days in year in the Dangi calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "dangi";
const options = { overflow: "reject" };

const sampleData = {
  1969: 354,
  1970: 355,
  1971: 384,
  1972: 354,
  1973: 354,
  1974: 384,
  1975: 354,
  1976: 384,
  1977: 354,
  1978: 355,
  1979: 384,
  1980: 355,
  1981: 354,
  1982: 384,
  1983: 354,
  1984: 384,
  1985: 354,
  1986: 354,
  1987: 385,
  1988: 354,
  1989: 355,
  1990: 384,
  1991: 354,
  1992: 354,
  1993: 383,
  1994: 355,
  1995: 384,
  1996: 355,
  1997: 354,
  1998: 384,
  1999: 354,
  2000: 354,
  2001: 384,
  2002: 354,
  2003: 355,
  2004: 384,
  2005: 354,
  2006: 385,
  2007: 354,
  2008: 354,
  2009: 384,
  2010: 354,
  2011: 354,
  2012: 384,
  2013: 355,
  2014: 384,
  2015: 354,
  2016: 355,
  2017: 384,
  2018: 354,
  2019: 354,
  2020: 384,
  2021: 354,
  2022: 355,
  2023: 384,
  2024: 354,
  2025: 384,
  2026: 355,
  2027: 354,
  2028: 383,
  2029: 355,
  2030: 354,
  2031: 384,
  2032: 355,
  2033: 384,
  2034: 354,
  2035: 354,
  2036: 384,
  2037: 354,
  2038: 354,
  2039: 384,
  2040: 355,
  2041: 355,
  2042: 384,
  2043: 354,
  2044: 384,
  2045: 354,
  2046: 354,
  2047: 384,
  2048: 354,
}

for (var [year, days] of Object.entries(sampleData)) {
    const date = Temporal.PlainDate.from({
        year,
        month: 1,
        calendar, day: 1
    });
    assert.sameValue(date.daysInYear, days);
}
