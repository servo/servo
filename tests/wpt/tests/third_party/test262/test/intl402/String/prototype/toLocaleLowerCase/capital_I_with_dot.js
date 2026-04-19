// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Check if String.prototype.toLocaleLowerCase supports mappings defined in SpecialCasings
info: |
    The result must be derived according to the case mappings in the Unicode character database (this explicitly
    includes not only the UnicodeData.txt file, but also the SpecialCasings.txt file that accompanies it).
es5id: 15.5.4.16
es6id: 21.1.3.20
---*/

// Locale-sensitive for Turkish and Azeri.
assert.sameValue("\u0130".toLocaleLowerCase("und"), "\u0069\u0307", "LATIN CAPITAL LETTER I WITH DOT ABOVE");
