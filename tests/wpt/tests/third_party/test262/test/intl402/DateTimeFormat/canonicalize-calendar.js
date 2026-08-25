// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat
description: Calendar IDs are canonicalized
locale: [en, en-u-ca-islamic-civil]
---*/

const fmt1 = new Intl.DateTimeFormat("en", { calendar: "islamicc" });
assert.sameValue(fmt1.resolvedOptions().calendar, "islamic-civil", "calendar ID is canonicalized (option)");

const fmt2 = new Intl.DateTimeFormat("en-u-ca-islamicc");
assert.sameValue(fmt1.resolvedOptions().calendar, "islamic-civil", "calendar ID is canonicalized (locale key)");

const fmt3 = new Intl.DateTimeFormat("en", { calendar: "ISO8601" });
assert.sameValue(fmt3.resolvedOptions().calendar, "iso8601", "calendar ID is lowercased");

assert.throws(
  RangeError,
  () => new Intl.DateTimeFormat("en", { calendar: "\u0130SO8601" }),
  "calendar ID is capital dotted I is not lowercased (first argument)"
);
