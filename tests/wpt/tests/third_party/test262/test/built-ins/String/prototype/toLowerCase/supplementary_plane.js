// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: String.prototype.toLowerCase() iterates over code points
info: |
    21.1.3.22 String.prototype.toLowerCase ( )

    ...
    4. Let cpList be a List containing in order the code points as defined in
       6.1.4 of S, starting at the first element of S.
    5. For each code point c in cpList, if the Unicode Character Database
       provides a language insensitive lower case equivalent of c then replace
       c in cpList with that equivalent code point(s).
es6id: 21.1.3.22
---*/

assert.sameValue("\uD801\uDC00".toLowerCase(), "\uD801\uDC28", "DESERET CAPITAL LETTER LONG I");
assert.sameValue("\uD801\uDC01".toLowerCase(), "\uD801\uDC29", "DESERET CAPITAL LETTER LONG E");
assert.sameValue("\uD801\uDC02".toLowerCase(), "\uD801\uDC2A", "DESERET CAPITAL LETTER LONG A");
assert.sameValue("\uD801\uDC03".toLowerCase(), "\uD801\uDC2B", "DESERET CAPITAL LETTER LONG AH");
assert.sameValue("\uD801\uDC04".toLowerCase(), "\uD801\uDC2C", "DESERET CAPITAL LETTER LONG O");
assert.sameValue("\uD801\uDC05".toLowerCase(), "\uD801\uDC2D", "DESERET CAPITAL LETTER LONG OO");
assert.sameValue("\uD801\uDC06".toLowerCase(), "\uD801\uDC2E", "DESERET CAPITAL LETTER SHORT I");
assert.sameValue("\uD801\uDC07".toLowerCase(), "\uD801\uDC2F", "DESERET CAPITAL LETTER SHORT E");
assert.sameValue("\uD801\uDC08".toLowerCase(), "\uD801\uDC30", "DESERET CAPITAL LETTER SHORT A");
assert.sameValue("\uD801\uDC09".toLowerCase(), "\uD801\uDC31", "DESERET CAPITAL LETTER SHORT AH");
assert.sameValue("\uD801\uDC0A".toLowerCase(), "\uD801\uDC32", "DESERET CAPITAL LETTER SHORT O");
assert.sameValue("\uD801\uDC0B".toLowerCase(), "\uD801\uDC33", "DESERET CAPITAL LETTER SHORT OO");
assert.sameValue("\uD801\uDC0C".toLowerCase(), "\uD801\uDC34", "DESERET CAPITAL LETTER AY");
assert.sameValue("\uD801\uDC0D".toLowerCase(), "\uD801\uDC35", "DESERET CAPITAL LETTER OW");
assert.sameValue("\uD801\uDC0E".toLowerCase(), "\uD801\uDC36", "DESERET CAPITAL LETTER WU");
assert.sameValue("\uD801\uDC0F".toLowerCase(), "\uD801\uDC37", "DESERET CAPITAL LETTER YEE");
assert.sameValue("\uD801\uDC10".toLowerCase(), "\uD801\uDC38", "DESERET CAPITAL LETTER H");
assert.sameValue("\uD801\uDC11".toLowerCase(), "\uD801\uDC39", "DESERET CAPITAL LETTER PEE");
assert.sameValue("\uD801\uDC12".toLowerCase(), "\uD801\uDC3A", "DESERET CAPITAL LETTER BEE");
assert.sameValue("\uD801\uDC13".toLowerCase(), "\uD801\uDC3B", "DESERET CAPITAL LETTER TEE");
assert.sameValue("\uD801\uDC14".toLowerCase(), "\uD801\uDC3C", "DESERET CAPITAL LETTER DEE");
assert.sameValue("\uD801\uDC15".toLowerCase(), "\uD801\uDC3D", "DESERET CAPITAL LETTER CHEE");
assert.sameValue("\uD801\uDC16".toLowerCase(), "\uD801\uDC3E", "DESERET CAPITAL LETTER JEE");
assert.sameValue("\uD801\uDC17".toLowerCase(), "\uD801\uDC3F", "DESERET CAPITAL LETTER KAY");
assert.sameValue("\uD801\uDC18".toLowerCase(), "\uD801\uDC40", "DESERET CAPITAL LETTER GAY");
assert.sameValue("\uD801\uDC19".toLowerCase(), "\uD801\uDC41", "DESERET CAPITAL LETTER EF");
assert.sameValue("\uD801\uDC1A".toLowerCase(), "\uD801\uDC42", "DESERET CAPITAL LETTER VEE");
assert.sameValue("\uD801\uDC1B".toLowerCase(), "\uD801\uDC43", "DESERET CAPITAL LETTER ETH");
assert.sameValue("\uD801\uDC1C".toLowerCase(), "\uD801\uDC44", "DESERET CAPITAL LETTER THEE");
assert.sameValue("\uD801\uDC1D".toLowerCase(), "\uD801\uDC45", "DESERET CAPITAL LETTER ES");
assert.sameValue("\uD801\uDC1E".toLowerCase(), "\uD801\uDC46", "DESERET CAPITAL LETTER ZEE");
assert.sameValue("\uD801\uDC1F".toLowerCase(), "\uD801\uDC47", "DESERET CAPITAL LETTER ESH");
assert.sameValue("\uD801\uDC20".toLowerCase(), "\uD801\uDC48", "DESERET CAPITAL LETTER ZHEE");
assert.sameValue("\uD801\uDC21".toLowerCase(), "\uD801\uDC49", "DESERET CAPITAL LETTER ER");
assert.sameValue("\uD801\uDC22".toLowerCase(), "\uD801\uDC4A", "DESERET CAPITAL LETTER EL");
assert.sameValue("\uD801\uDC23".toLowerCase(), "\uD801\uDC4B", "DESERET CAPITAL LETTER EM");
assert.sameValue("\uD801\uDC24".toLowerCase(), "\uD801\uDC4C", "DESERET CAPITAL LETTER EN");
assert.sameValue("\uD801\uDC25".toLowerCase(), "\uD801\uDC4D", "DESERET CAPITAL LETTER ENG");
assert.sameValue("\uD801\uDC26".toLowerCase(), "\uD801\uDC4E", "DESERET CAPITAL LETTER OI");
assert.sameValue("\uD801\uDC27".toLowerCase(), "\uD801\uDC4F", "DESERET CAPITAL LETTER EW");
