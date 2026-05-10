// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- lead and trail pattern in RegExpUnicodeEscapeSequence in CharacterClass.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// ==== standalone ====

assert.compareArray(/[\uD83D\uDC38]/u.exec("\uD83D\uDC38"),
              ["\uD83D\uDC38"]);
assert.sameValue(/[\uD83D\uDC38]/u.exec("\uD83D"),
         null);
assert.sameValue(/[\uD83D\uDC38]/u.exec("\uDC38"),
         null);

// no unicode flag
assert.compareArray(/[\uD83D\uDC38]/.exec("\uD83D\uDC38"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D\uDC38]/.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D\uDC38]/.exec("\uDC38"),
              ["\uDC38"]);

// RegExp constructor
assert.compareArray(new RegExp("[\uD83D\uDC38]", "u").exec("\uD83D\uDC38"),
              ["\uD83D\uDC38"]);
assert.sameValue(new RegExp("[\uD83D\uDC38]", "u").exec("\uD83D"),
         null);
assert.sameValue(new RegExp("[\uD83D\uDC38]", "u").exec("\uDC38"),
         null);

// RegExp constructor, no unicode flag
assert.compareArray(new RegExp("[\uD83D\uDC38]", "").exec("\uD83D\uDC38"),
              ["\uD83D"]);
assert.compareArray(new RegExp("[\uD83D\uDC38]", "").exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(new RegExp("[\uD83D\uDC38]", "").exec("\uDC38"),
              ["\uDC38"]);

// ==== lead-only ====

// match only non-surrogate pair
assert.compareArray(/[\uD83D]/u.exec("\uD83D\uDBFF"),
              ["\uD83D"]);
assert.sameValue(/[\uD83D]/u.exec("\uD83D\uDC00"),
         null);
assert.sameValue(/[\uD83D]/u.exec("\uD83D\uDFFF"),
         null);
assert.compareArray(/[\uD83D]/u.exec("\uD83D\uE000"),
              ["\uD83D"]);

// match before non-tail char
assert.compareArray(/[\uD83D]/u.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D]/u.exec("\uD83DA"),
              ["\uD83D"]);

// no unicode flag
assert.compareArray(/[\uD83D]/.exec("\uD83D\uDBFF"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D]/.exec("\uD83D\uDC00"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D]/.exec("\uD83D\uDFFF"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D]/.exec("\uD83D\uE000"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D]/.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D]/.exec("\uD83DA"),
              ["\uD83D"]);

// ==== trail-only ====

// match only non-surrogate pair
assert.compareArray(/[\uDC38]/u.exec("\uD7FF\uDC38"),
              ["\uDC38"]);
assert.sameValue(/[\uDC38]/u.exec("\uD800\uDC38"),
         null);
assert.sameValue(/[\uDC38]/u.exec("\uDBFF\uDC38"),
         null);
assert.compareArray(/[\uDC38]/u.exec("\uDC00\uDC38"),
              ["\uDC38"]);

// match after non-lead char
assert.compareArray(/[\uDC38]/u.exec("\uDC38"),
              ["\uDC38"]);
assert.compareArray(/[\uDC38]/u.exec("A\uDC38"),
              ["\uDC38"]);

// no unicode flag
assert.compareArray(/[\uDC38]/.exec("\uD7FF\uDC38"),
              ["\uDC38"]);
assert.compareArray(/[\uDC38]/.exec("\uD800\uDC38"),
              ["\uDC38"]);
assert.compareArray(/[\uDC38]/.exec("\uDBFF\uDC38"),
              ["\uDC38"]);
assert.compareArray(/[\uDC38]/.exec("\uDC00\uDC38"),
              ["\uDC38"]);
assert.compareArray(/[\uDC38]/.exec("\uDC38"),
              ["\uDC38"]);
assert.compareArray(/[\uDC38]/.exec("A\uDC38"),
              ["\uDC38"]);

// ==== invalid trail ====

assert.compareArray(/[\uD83D\u3042]*/u.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D\u3042]*/u.exec("\uD83D\u3042"),
              ["\uD83D\u3042"]);
assert.compareArray(/[\uD83D\u3042]*/u.exec("\uD83D\u3042\u3042\uD83D"),
              ["\uD83D\u3042\u3042\uD83D"]);

assert.compareArray(/[\uD83D\u{3042}]*/u.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[\uD83D\u{3042}]*/u.exec("\uD83D\u3042"),
              ["\uD83D\u3042"]);
assert.compareArray(/[\uD83D\u{3042}]*/u.exec("\uD83D\u3042\u3042\uD83D"),
              ["\uD83D\u3042\u3042\uD83D"]);

assert.compareArray(/[\uD83DA]*/u.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[\uD83DA]*/u.exec("\uD83DA"),
              ["\uD83DA"]);
assert.compareArray(/[\uD83DA]*/u.exec("\uD83DAA\uD83D"),
              ["\uD83DAA\uD83D"]);

// ==== wrong patterns ====

assert.throws(SyntaxError, () => eval(`/[\\u]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u0]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u00]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u000]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u000G]/u`));
assert.throws(SyntaxError, () => eval(`/[\\u0.00]/u`));
assert.throws(SyntaxError, () => eval(`/[\\uD83D\\u]/u`));
assert.throws(SyntaxError, () => eval(`/[\\uD83D\\u0]/u`));
assert.throws(SyntaxError, () => eval(`/[\\uD83D\\u00]/u`));
assert.throws(SyntaxError, () => eval(`/[\\uD83D\\u000]/u`));
assert.throws(SyntaxError, () => eval(`/[\\uD83D\\u000G]/u`));
assert.throws(SyntaxError, () => eval(`/[\\uD83D\\u0.00]/u`));
