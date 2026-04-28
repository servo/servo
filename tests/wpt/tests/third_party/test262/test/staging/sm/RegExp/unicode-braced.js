// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- braced pattern in RegExpUnicodeEscapeSequence.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// ==== standalone ====

assert.compareArray(/\u{41}/u.exec("ABC"),
              ["A"]);
assert.compareArray(/\u{41}/.exec("ABC" + "u".repeat(41)),
              ["u".repeat(41)]);

assert.compareArray(/\u{4A}/u.exec("JKL"),
              ["J"]);
assert.compareArray(/\u{4A}/.exec("JKLu{4A}"),
              ["u{4A}"]);

assert.compareArray(/\u{1F438}/u.exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/\u{1F438}/.exec("u{1F438}"),
              ["u{1F438}"]);

assert.compareArray(/\u{0}/u.exec("\u{0}"),
              ["\u{0}"]);
assert.compareArray(/\u{10FFFF}/u.exec("\u{10FFFF}"),
              ["\u{10FFFF}"]);
assert.compareArray(/\u{10ffff}/u.exec("\u{10FFFF}"),
              ["\u{10FFFF}"]);

// leading 0
assert.compareArray(/\u{0000000000000000000000}/u.exec("\u{0}"),
              ["\u{0}"]);
assert.compareArray(/\u{000000000000000010FFFF}/u.exec("\u{10FFFF}"),
              ["\u{10FFFF}"]);

// RegExp constructor
assert.compareArray(new RegExp("\\u{0}", "u").exec("\u{0}"),
              ["\u{0}"]);
assert.compareArray(new RegExp("\\u{41}", "u").exec("ABC"),
              ["A"]);
assert.compareArray(new RegExp("\\u{1F438}", "u").exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(new RegExp("\\u{10FFFF}", "u").exec("\u{10FFFF}"),
              ["\u{10FFFF}"]);

assert.compareArray(new RegExp("\\u{0000000000000000}", "u").exec("\u{0}"),
              ["\u{0}"]);

assert.compareArray(eval(`/\\u{${"0".repeat(Math.pow(2, 24)) + "1234"}}/u`).exec("\u{1234}"),
              ["\u{1234}"]);
assert.compareArray(new RegExp(`\\u{${"0".repeat(Math.pow(2, 24)) + "1234"}}`, "u").exec("\u{1234}"),
              ["\u{1234}"]);

// ==== ? ====

assert.compareArray(/\u{1F438}?/u.exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/\u{1F438}?/u.exec(""),
              [""]);

// lead-only target
assert.compareArray(/\u{1F438}?/u.exec("\uD83D"),
              [""]);

// RegExp constructor
assert.compareArray(new RegExp("\\u{1F438}?", "u").exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(new RegExp("\\u{1F438}?", "u").exec(""),
              [""]);
assert.compareArray(new RegExp("\\u{1F438}?", "u").exec("\uD83D"),
              [""]);

// ==== + ====

assert.compareArray(/\u{1F438}+/u.exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/\u{1F438}+/u.exec("\u{1F438}\u{1F438}"),
              ["\u{1F438}\u{1F438}"]);
assert.sameValue(/\u{1F438}+/u.exec(""),
         null);

// lead-only target
assert.sameValue(/\u{1F438}+/u.exec("\uD83D"),
         null);
assert.compareArray(/\u{1F438}+/u.exec("\uD83D\uDC38\uDC38"),
              ["\uD83D\uDC38"]);

// ==== * ====

assert.compareArray(/\u{1F438}*/u.exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/\u{1F438}*/u.exec("\u{1F438}\u{1F438}"),
              ["\u{1F438}\u{1F438}"]);
assert.compareArray(/\u{1F438}*/u.exec(""),
              [""]);

// lead-only target
assert.compareArray(/\u{1F438}*/u.exec("\uD83D"),
              [""]);
assert.compareArray(/\u{1F438}*/u.exec("\uD83D\uDC38\uDC38"),
              ["\uD83D\uDC38"]);

// ==== lead-only ====

// match only non-surrogate pair
assert.compareArray(/\u{D83D}/u.exec("\uD83D\uDBFF"),
              ["\uD83D"]);
assert.sameValue(/\u{D83D}/u.exec("\uD83D\uDC00"),
         null);
assert.sameValue(/\u{D83D}/u.exec("\uD83D\uDFFF"),
         null);
assert.compareArray(/\u{D83D}/u.exec("\uD83D\uE000"),
              ["\uD83D"]);

// match before non-tail char
assert.compareArray(/\u{D83D}/u.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/\u{D83D}/u.exec("\uD83DA"),
              ["\uD83D"]);

// ==== trail-only ====

// match only non-surrogate pair
assert.compareArray(/\u{DC38}/u.exec("\uD7FF\uDC38"),
              ["\uDC38"]);
assert.sameValue(/\u{DC38}/u.exec("\uD800\uDC38"),
         null);
assert.sameValue(/\u{DC38}/u.exec("\uDBFF\uDC38"),
         null);
assert.compareArray(/\u{DC38}/u.exec("\uDC00\uDC38"),
              ["\uDC38"]);

// match after non-lead char
assert.compareArray(/\u{DC38}/u.exec("\uDC38"),
              ["\uDC38"]);
assert.compareArray(/\u{DC38}/u.exec("A\uDC38"),
              ["\uDC38"]);

// ==== wrong patterns ====

assert.throws(SyntaxError, () => eval(`/\\u{-1}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{0.0}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{G}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{{/u`));
assert.throws(SyntaxError, () => eval(`/\\u{/u`));
assert.throws(SyntaxError, () => eval(`/\\u{110000}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{00110000}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{100000000000000000000000000000}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{   FFFF}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{FFFF   }/u`));
assert.throws(SyntaxError, () => eval(`/\\u{FF   FF}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{F F F F}/u`));
assert.throws(SyntaxError, () => eval(`/\\u{100000001}/u`));

// surrogate pair with braced
assert.sameValue(/\u{D83D}\u{DC38}+/u.exec("\uD83D\uDC38\uDC38"),
         null);
assert.sameValue(/\uD83D\u{DC38}+/u.exec("\uD83D\uDC38\uDC38"),
         null);
assert.sameValue(/\u{D83D}\uDC38+/u.exec("\uD83D\uDC38\uDC38"),
         null);
