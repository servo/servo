// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- braced pattern in RegExpUnicodeEscapeSequence in CharacterClass.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// ==== standalone ====

assert.compareArray(/[\u{41}]/u.exec("ABC"),
              ["A"]);

assert.compareArray(/[\u{1F438}]/u.exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.sameValue(/[\u{1F438}]/u.exec("\uD83D"),
         null);
assert.sameValue(/[\u{1F438}]/u.exec("\uDC38"),
         null);

assert.compareArray(/[\u{0}]/u.exec("\u{0}"),
              ["\u{0}"]);
assert.compareArray(/[\u{10FFFF}]/u.exec("\u{10FFFF}"),
              ["\u{10FFFF}"]);
assert.compareArray(/[\u{10ffff}]/u.exec("\u{10FFFF}"),
              ["\u{10FFFF}"]);

// leading 0
assert.compareArray(/[\u{0000000000000000000000}]/u.exec("\u{0}"),
              ["\u{0}"]);
assert.compareArray(/[\u{000000000000000010FFFF}]/u.exec("\u{10FFFF}"),
              ["\u{10FFFF}"]);

// RegExp constructor
assert.compareArray(new RegExp("[\\u{0}]", "u").exec("\u{0}"),
              ["\u{0}"]);
assert.compareArray(new RegExp("[\\u{41}]", "u").exec("ABC"),
              ["A"]);
assert.compareArray(new RegExp("[\\u{1F438}]", "u").exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(new RegExp("[\\u{10FFFF}]", "u").exec("\u{10FFFF}"),
              ["\u{10FFFF}"]);

assert.compareArray(new RegExp("[\\u{0000000000000000}]", "u").exec("\u{0}"),
              ["\u{0}"]);

assert.compareArray(eval(`/[\\u{${"0".repeat(Math.pow(2, 24)) + "1234"}}]/u`).exec("\u{1234}"),
              ["\u{1234}"]);
assert.compareArray(new RegExp(`[\\u{${"0".repeat(Math.pow(2, 24)) + "1234"}}]`, "u").exec("\u{1234}"),
              ["\u{1234}"]);

// ==== BMP + non-BMP ====

assert.compareArray(/[A\u{1F438}]/u.exec("A\u{1F438}"),
              ["A"]);
assert.compareArray(/[A\u{1F438}]/u.exec("\u{1F438}A"),
              ["\u{1F438}"]);

// lead-only target
assert.compareArray(/[A\u{1F438}]/u.exec("\uD83DA"),
              ["A"]);
assert.sameValue(/[A\u{1F438}]/u.exec("\uD83D"),
         null);

// +
assert.compareArray(/[A\u{1F438}]+/u.exec("\u{1F438}A\u{1F438}A"),
              ["\u{1F438}A\u{1F438}A"]);

// trail surrogate + lead surrogate
assert.compareArray(/[A\u{1F438}]+/u.exec("\uD83D\uDC38A\uDC38\uD83DA"),
              ["\uD83D\uDC38A"]);

// ==== non-BMP + non-BMP ====

assert.compareArray(/[\u{1F418}\u{1F438}]/u.exec("\u{1F418}\u{1F438}"),
              ["\u{1F418}"]);

assert.compareArray(/[\u{1F418}\u{1F438}]+/u.exec("\u{1F418}\u{1F438}"),
              ["\u{1F418}\u{1F438}"]);
assert.compareArray(/[\u{1F418}\u{1F438}]+/u.exec("\u{1F418}\uDC38\uD83D"),
              ["\u{1F418}"]);
assert.compareArray(/[\u{1F418}\u{1F438}]+/u.exec("\uDC18\uD83D\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/[\u{1F418}\u{1F438}]+/u.exec("\uDC18\u{1F438}\uD83D"),
              ["\u{1F438}"]);

// trail surrogate + lead surrogate
assert.sameValue(/[\u{1F418}\u{1F438}]+/u.exec("\uDC18\uDC38\uD83D\uD83D"),
         null);

// ==== non-BMP + non-BMP range (from_lead == to_lead) ====

assert.compareArray(/[\u{1F418}-\u{1F438}]/u.exec("\u{1F418}"),
              ["\u{1F418}"]);
assert.compareArray(/[\u{1F418}-\u{1F438}]/u.exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/[\u{1F418}-\u{1F438}]/u.exec("\u{1F427}"),
              ["\u{1F427}"]);

assert.sameValue(/[\u{1F418}-\u{1F438}]/u.exec("\u{1F417}"),
         null);
assert.sameValue(/[\u{1F418}-\u{1F438}]/u.exec("\u{1F439}"),
         null);

// ==== non-BMP + non-BMP range (from_lead + 1 == to_lead) ====

assert.compareArray(/[\u{1F17C}-\u{1F438}]/u.exec("\uD83C\uDD7C"),
              ["\uD83C\uDD7C"]);
assert.compareArray(/[\u{1F17C}-\u{1F438}]/u.exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/[\u{1F17C}-\u{1F438}]/u.exec("\uD83C\uDF99"),
              ["\uD83C\uDF99"]);
assert.compareArray(/[\u{1F17C}-\u{1F438}]/u.exec("\uD83D\uDC00"),
              ["\uD83D\uDC00"]);

assert.sameValue(/[\u{1F17C}-\u{1F438}]/u.exec("\uD83C\uDD7B"),
         null);
assert.sameValue(/[\u{1F17C}-\u{1F438}]/u.exec("\uD83C\uE000"),
         null);
assert.sameValue(/[\u{1F17C}-\u{1F438}]/u.exec("\uD83D\uDB99"),
         null);
assert.sameValue(/[\u{1F17C}-\u{1F438}]/u.exec("\uD83D\uDC39"),
         null);

// ==== non-BMP + non-BMP range (from_lead + 2 == to_lead) ====

assert.compareArray(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83C\uDD7C"),
              ["\uD83C\uDD7C"]);
assert.compareArray(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83E\uDC29"),
              ["\uD83E\uDC29"]);

assert.compareArray(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83C\uDF99"),
              ["\uD83C\uDF99"]);
assert.compareArray(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83D\uDC00"),
              ["\uD83D\uDC00"]);
assert.compareArray(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83D\uDF99"),
              ["\uD83D\uDF99"]);
assert.compareArray(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83E\uDC00"),
              ["\uD83E\uDC00"]);

assert.sameValue(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83C\uDD7B"),
         null);
assert.sameValue(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83C\uE000"),
         null);
assert.sameValue(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83D\uDB99"),
         null);
assert.sameValue(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83D\uE000"),
         null);
assert.sameValue(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83E\uDB99"),
         null);
assert.sameValue(/[\u{1F17C}-\u{1F829}]/u.exec("\uD83E\uDC30"),
         null);

// ==== non-BMP + non-BMP range (other) ====

assert.compareArray(/[\u{1D164}-\u{1F438}]/u.exec("\uD834\uDD64"),
              ["\uD834\uDD64"]);
assert.compareArray(/[\u{1D164}-\u{1F438}]/u.exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/[\u{1D164}-\u{1F438}]/u.exec("\uD836\uDF99"),
              ["\uD836\uDF99"]);
assert.compareArray(/[\u{1D164}-\u{1F438}]/u.exec("\uD838\uDC00"),
              ["\uD838\uDC00"]);

assert.sameValue(/[\u{1D164}-\u{1F438}]/u.exec("\uD834\uDD63"),
         null);
assert.sameValue(/[\u{1D164}-\u{1F438}]/u.exec("\uD83D\uDC39"),
         null);

assert.sameValue(/[\u{1D164}-\u{1F438}]/u.exec("\uD834\uE000"),
         null);
assert.sameValue(/[\u{1D164}-\u{1F438}]/u.exec("\uD835\uDB99"),
         null);
assert.sameValue(/[\u{1D164}-\u{1F438}]/u.exec("\uD83C\uE000"),
         null);
assert.sameValue(/[\u{1D164}-\u{1F438}]/u.exec("\uD83D\uDB99"),
         null);

// ==== BMP + non-BMP range ====

assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("B"),
              ["B"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("C"),
              ["C"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uFFFF"),
              ["\uFFFF"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uD800\uDC00"),
              ["\uD800\uDC00"]);

assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uD800"),
              ["\uD800"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uDBFF"),
              ["\uDBFF"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uDC00"),
              ["\uDC00"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uDFFF"),
              ["\uDFFF"]);

assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uDC38"),
              ["\uDC38"]);

assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uD83D\uDBFF"),
              ["\uD83D"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uD83D\uDC00"),
              ["\uD83D\uDC00"]);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uD83D\uDC38"),
              ["\uD83D\uDC38"]);
assert.sameValue(/[\u{42}-\u{1F438}]/u.exec("\uD83D\uDC39"),
         null);
assert.sameValue(/[\u{42}-\u{1F438}]/u.exec("\uD83D\uDFFF"),
         null);
assert.compareArray(/[\u{42}-\u{1F438}]/u.exec("\uD83D\uE000"),
              ["\uD83D"]);

assert.sameValue(/[\u{42}-\u{1F438}]/u.exec("A"),
         null);

// ==== wrong patterns ====

assert.throws(SyntaxError, () => eval(`/[\\u{-1}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{0.0}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{G}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{{]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{110000}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{00110000}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{100000000000000000000000000000}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{   FFFF}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{FFFF   }]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{FF   FF}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{F F F F}]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u{100000001}]/u`));
