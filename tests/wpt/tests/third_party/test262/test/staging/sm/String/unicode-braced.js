// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Add \\u{xxxxxx} string literals
info: bugzilla.mozilla.org/show_bug.cgi?id=320500
esid: pending
---*/

assert.sameValue("\u{0}", String.fromCodePoint(0x0));
assert.sameValue("\u{1}", String.fromCodePoint(0x1));
assert.sameValue("\u{10}", String.fromCodePoint(0x10));
assert.sameValue("\u{100}", String.fromCodePoint(0x100));
assert.sameValue("\u{1000}", String.fromCodePoint(0x1000));
assert.sameValue("\u{D7FF}", String.fromCodePoint(0xD7FF));
assert.sameValue("\u{D800}", String.fromCodePoint(0xD800));
assert.sameValue("\u{DBFF}", String.fromCodePoint(0xDBFF));
assert.sameValue("\u{DC00}", String.fromCodePoint(0xDC00));
assert.sameValue("\u{DFFF}", String.fromCodePoint(0xDFFF));
assert.sameValue("\u{E000}", String.fromCodePoint(0xE000));
assert.sameValue("\u{10000}", String.fromCodePoint(0x10000));
assert.sameValue("\u{100000}", String.fromCodePoint(0x100000));
assert.sameValue("\u{10FFFF}", String.fromCodePoint(0x10FFFF));
assert.sameValue("\u{10ffff}", String.fromCodePoint(0x10FFFF));

assert.sameValue("A\u{1}\u{10}B\u{100}\u{1000}\u{10000}C\u{100000}",
         "A" +
         String.fromCodePoint(0x1) +
         String.fromCodePoint(0x10) +
         "B" +
         String.fromCodePoint(0x100) +
         String.fromCodePoint(0x1000) +
         String.fromCodePoint(0x10000) +
         "C" +
         String.fromCodePoint(0x100000));

assert.sameValue('\u{10ffff}', String.fromCodePoint(0x10FFFF));
assert.sameValue(`\u{10ffff}`, String.fromCodePoint(0x10FFFF));
assert.sameValue(`\u{10ffff}${""}`, String.fromCodePoint(0x10FFFF));
assert.sameValue(`${""}\u{10ffff}`, String.fromCodePoint(0x10FFFF));
assert.sameValue(`${""}\u{10ffff}${""}`, String.fromCodePoint(0x10FFFF));

assert.sameValue("\u{00}", String.fromCodePoint(0x0));
assert.sameValue("\u{00000000000000000}", String.fromCodePoint(0x0));
assert.sameValue("\u{00000000000001000}", String.fromCodePoint(0x1000));

assert.sameValue(eval(`"\\u{${"0".repeat(Math.pow(2, 24)) + "1234"}}"`), String.fromCodePoint(0x1234));

assert.sameValue("\U{0}", "U{0}");

assert.throws(SyntaxError, () => eval(`"\\u{-1}"`));
assert.throws(SyntaxError, () => eval(`"\\u{0.0}"`));
assert.throws(SyntaxError, () => eval(`"\\u{G}"`));
assert.throws(SyntaxError, () => eval(`"\\u{}"`));
assert.throws(SyntaxError, () => eval(`"\\u{{"`));
assert.throws(SyntaxError, () => eval(`"\\u{"`));
assert.throws(SyntaxError, () => eval(`"\\u{110000}"`));
assert.throws(SyntaxError, () => eval(`"\\u{00110000}"`));
assert.throws(SyntaxError, () => eval(`"\\u{100000000000000000000000000000}"`));
assert.throws(SyntaxError, () => eval(`"\\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}"`));
assert.throws(SyntaxError, () => eval(`"\\u{   FFFF}"`));
assert.throws(SyntaxError, () => eval(`"\\u{FFFF   }"`));
assert.throws(SyntaxError, () => eval(`"\\u{FF   FF}"`));
assert.throws(SyntaxError, () => eval(`"\\u{F F F F}"`));
assert.throws(SyntaxError, () => eval(`"\\u{100000001}"`));
