// Copyright (C) 2015 Zirak. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: An incomplete HexEscape or UnicodeEscape should be treated as an Identity Escape
info: |
    An incomplete HexEscape (e.g. /\x/) or UnicodeEscape (/\u/) should fall
    through to IdentityEscape
esid: prod-AtomEscape
---*/

// Hex escape
assert(/\x/.test("x"), "/\\x/");
assert(/\xa/.test("xa"), "/\\xa/");

// Unicode escape
assert(/\u/.test("u"), "/\\u/");
assert(/\ua/.test("ua"), "/\\ua/");
