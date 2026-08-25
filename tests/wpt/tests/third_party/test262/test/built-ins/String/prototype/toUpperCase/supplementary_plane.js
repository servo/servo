// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String.prototype.toUpperCase() iterates over code points
info: |
    21.1.3.24 String.prototype.toUpperCase ( )

    This function behaves in exactly the same way as String.prototype.toLowerCase,
    except that code points are mapped to their uppercase equivalents as specified
    in the Unicode Character Database.

    21.1.3.22 String.prototype.toLowerCase ( )

    ...
    4. Let cpList be a List containing in order the code points as defined in
       6.1.4 of S, starting at the first element of S.
    5. For each code point c in cpList, if the Unicode Character Database
       provides a language insensitive lower case equivalent of c then replace
       c in cpList with that equivalent code point(s).
es6id: 21.1.3.24
---*/

assert.sameValue("\uD801\uDC28".toUpperCase(), "\uD801\uDC00", "DESERET SMALL LETTER LONG I");
assert.sameValue("\uD801\uDC29".toUpperCase(), "\uD801\uDC01", "DESERET SMALL LETTER LONG E");
assert.sameValue("\uD801\uDC2A".toUpperCase(), "\uD801\uDC02", "DESERET SMALL LETTER LONG A");
assert.sameValue("\uD801\uDC2B".toUpperCase(), "\uD801\uDC03", "DESERET SMALL LETTER LONG AH");
assert.sameValue("\uD801\uDC2C".toUpperCase(), "\uD801\uDC04", "DESERET SMALL LETTER LONG O");
assert.sameValue("\uD801\uDC2D".toUpperCase(), "\uD801\uDC05", "DESERET SMALL LETTER LONG OO");
assert.sameValue("\uD801\uDC2E".toUpperCase(), "\uD801\uDC06", "DESERET SMALL LETTER SHORT I");
assert.sameValue("\uD801\uDC2F".toUpperCase(), "\uD801\uDC07", "DESERET SMALL LETTER SHORT E");
assert.sameValue("\uD801\uDC30".toUpperCase(), "\uD801\uDC08", "DESERET SMALL LETTER SHORT A");
assert.sameValue("\uD801\uDC31".toUpperCase(), "\uD801\uDC09", "DESERET SMALL LETTER SHORT AH");
assert.sameValue("\uD801\uDC32".toUpperCase(), "\uD801\uDC0A", "DESERET SMALL LETTER SHORT O");
assert.sameValue("\uD801\uDC33".toUpperCase(), "\uD801\uDC0B", "DESERET SMALL LETTER SHORT OO");
assert.sameValue("\uD801\uDC34".toUpperCase(), "\uD801\uDC0C", "DESERET SMALL LETTER AY");
assert.sameValue("\uD801\uDC35".toUpperCase(), "\uD801\uDC0D", "DESERET SMALL LETTER OW");
assert.sameValue("\uD801\uDC36".toUpperCase(), "\uD801\uDC0E", "DESERET SMALL LETTER WU");
assert.sameValue("\uD801\uDC37".toUpperCase(), "\uD801\uDC0F", "DESERET SMALL LETTER YEE");
assert.sameValue("\uD801\uDC38".toUpperCase(), "\uD801\uDC10", "DESERET SMALL LETTER H");
assert.sameValue("\uD801\uDC39".toUpperCase(), "\uD801\uDC11", "DESERET SMALL LETTER PEE");
assert.sameValue("\uD801\uDC3A".toUpperCase(), "\uD801\uDC12", "DESERET SMALL LETTER BEE");
assert.sameValue("\uD801\uDC3B".toUpperCase(), "\uD801\uDC13", "DESERET SMALL LETTER TEE");
assert.sameValue("\uD801\uDC3C".toUpperCase(), "\uD801\uDC14", "DESERET SMALL LETTER DEE");
assert.sameValue("\uD801\uDC3D".toUpperCase(), "\uD801\uDC15", "DESERET SMALL LETTER CHEE");
assert.sameValue("\uD801\uDC3E".toUpperCase(), "\uD801\uDC16", "DESERET SMALL LETTER JEE");
assert.sameValue("\uD801\uDC3F".toUpperCase(), "\uD801\uDC17", "DESERET SMALL LETTER KAY");
assert.sameValue("\uD801\uDC40".toUpperCase(), "\uD801\uDC18", "DESERET SMALL LETTER GAY");
assert.sameValue("\uD801\uDC41".toUpperCase(), "\uD801\uDC19", "DESERET SMALL LETTER EF");
assert.sameValue("\uD801\uDC42".toUpperCase(), "\uD801\uDC1A", "DESERET SMALL LETTER VEE");
assert.sameValue("\uD801\uDC43".toUpperCase(), "\uD801\uDC1B", "DESERET SMALL LETTER ETH");
assert.sameValue("\uD801\uDC44".toUpperCase(), "\uD801\uDC1C", "DESERET SMALL LETTER THEE");
assert.sameValue("\uD801\uDC45".toUpperCase(), "\uD801\uDC1D", "DESERET SMALL LETTER ES");
assert.sameValue("\uD801\uDC46".toUpperCase(), "\uD801\uDC1E", "DESERET SMALL LETTER ZEE");
assert.sameValue("\uD801\uDC47".toUpperCase(), "\uD801\uDC1F", "DESERET SMALL LETTER ESH");
assert.sameValue("\uD801\uDC48".toUpperCase(), "\uD801\uDC20", "DESERET SMALL LETTER ZHEE");
assert.sameValue("\uD801\uDC49".toUpperCase(), "\uD801\uDC21", "DESERET SMALL LETTER ER");
assert.sameValue("\uD801\uDC4A".toUpperCase(), "\uD801\uDC22", "DESERET SMALL LETTER EL");
assert.sameValue("\uD801\uDC4B".toUpperCase(), "\uD801\uDC23", "DESERET SMALL LETTER EM");
assert.sameValue("\uD801\uDC4C".toUpperCase(), "\uD801\uDC24", "DESERET SMALL LETTER EN");
assert.sameValue("\uD801\uDC4D".toUpperCase(), "\uD801\uDC25", "DESERET SMALL LETTER ENG");
assert.sameValue("\uD801\uDC4E".toUpperCase(), "\uD801\uDC26", "DESERET SMALL LETTER OI");
assert.sameValue("\uD801\uDC4F".toUpperCase(), "\uD801\uDC27", "DESERET SMALL LETTER EW");
