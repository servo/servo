// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-ClassEscapes
description: >
  \b escape inside CharacterClass is valid in Unicode patterns (unlike \B).
info: |
  ClassEscape[U] ::
    b

  Static Semantics: CharacterValue

  ClassEscape :: b

  1. Return the code point value of U+0008 (BACKSPACE).
---*/

assert(/[\b]/u.test('\u0008'));
assert(/[\b-A]/u.test('A'));
