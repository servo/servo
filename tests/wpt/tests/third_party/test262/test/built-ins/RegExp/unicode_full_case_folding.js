// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-canonicalize-ch
description: >
  Case-insensitive Unicode RegExps should not apply full case folding mappings
info: |
  Canonicalize ( _rer_, _ch_ )
  1. If _rer_.[[Unicode]] is *true* and _rer_.[[IgnoreCase]] is *true*, then
    a. If the file `CaseFolding.txt` of the Unicode Character Database provides
      a simple or common case folding mapping for _ch_, return the result of
      applying that mapping to _ch_.
    b. Return _ch_.

  See https://unicode.org/Public/UCD/latest/ucd/CaseFolding.txt for the case
  folding mappings.
---*/

assert(/[\u0390]/ui.test("\u1fd3"), "\\u0390 does not match \\u1fd3");
assert(/[\u1fd3]/ui.test("\u0390"), "\\u1fd3 does not match \\u0390");
assert(/[\u03b0]/ui.test("\u1fe3"), "\\u03b0 does not match \\u1fe3");
assert(/[\u1fe3]/ui.test("\u03b0"), "\\u1fe3 does not match \\u03b0");
assert(/[\ufb05]/ui.test("\ufb06"), "\\ufb05 does not match \\ufb06");
assert(/[\ufb06]/ui.test("\ufb05"), "\\ufb06 does not match \\ufb05");
