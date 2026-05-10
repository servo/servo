// Copyright (C) 2017 Leonardo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-CharacterClassEscape
description: >
  U+180E is no longer a Unicode `Space_Separator` symbol as of Unicode v6.3.0.
info: |
  21.2.2.12 CharacterClassEscape

  ...

  The production CharacterClassEscape::s evaluates as follows:

  Return the set of characters containing the characters that are on the
  right-hand side of the WhiteSpace or LineTerminator productions.

  The production CharacterClassEscape::S evaluates as follows:

  Return the set of all characters not included in the set returned by
  CharacterClassEscape::s .
features: [u180e]
---*/

assert.sameValue("\u180E".replace(/\s+/g, "42"), "\u180E", "\\s should not match U+180E");
assert.sameValue("\u180E".replace(/\S+/g, "42"), "42", "\\S matches U+180E");
