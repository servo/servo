// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Checks the order of evaluations of arguments and options for the Locale
    constructor.
features: [Intl.Locale]
includes: [compareArray.js]
---*/

const order = [];
function handleGet(name, value) {
  order.push("get " + name);
  return {
    toString() {
      order.push("toString " + name);
      return value;
    }
  };
}
new Intl.Locale(
  { toString() { order.push("tag toString"); return "en"; } },
  {
    get calendar() {
      return handleGet("calendar", "gregory");
    },
    get caseFirst() {
      return handleGet("caseFirst", "upper");
    },
    get collation() {
      return handleGet("collation", "zhuyin");
    },
    get hourCycle() {
      return handleGet("hourCycle", "h24");
    },
    get language() {
      return handleGet("language", "de");
    },
    get numberingSystem() {
      return handleGet("numberingSystem", "latn");
    },
    get numeric() {
      order.push("get numeric");
      return false;
    },
    get region() {
      return handleGet("region", "DE");
    },
    get script() {
      return handleGet("script", "Latn");
    },
    get variants() {
      return handleGet("variants", "fonipa-1996");
    },
  }
);

const expected_order = [
  "tag toString",
  "get language",
  "toString language",
  "get script",
  "toString script",
  "get region",
  "toString region",
  "get variants",
  "toString variants",
  "get calendar",
  "toString calendar",
  "get collation",
  "toString collation",
  "get hourCycle",
  "toString hourCycle",
  "get caseFirst",
  "toString caseFirst",
  "get numeric",
  "get numberingSystem",
  "toString numberingSystem"
];

assert.compareArray(order, expected_order);
