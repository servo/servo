// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Check if String.prototype.toLocaleUpperCase supports mappings defined in SpecialCasings
info: |
    The result must be derived according to the locale-insensitive case mappings in the Unicode Character
    Database (this explicitly includes not only the UnicodeData.txt file, but also all locale-insensitive
    mappings in the SpecialCasings.txt file that accompanies it).
es5id: 15.5.4.19
es6id: 21.1.3.21
---*/

// SpecialCasing.txt, except for conditional mappings.

assert.sameValue("\u00DF".toLocaleUpperCase(), "\u0053\u0053", "LATIN SMALL LETTER SHARP S");

assert.sameValue("\u0130".toLocaleUpperCase(), "\u0130", "LATIN CAPITAL LETTER I WITH DOT ABOVE");

assert.sameValue("\uFB00".toLocaleUpperCase(), "\u0046\u0046", "LATIN SMALL LIGATURE FF");
assert.sameValue("\uFB01".toLocaleUpperCase(), "\u0046\u0049", "LATIN SMALL LIGATURE FI");
assert.sameValue("\uFB02".toLocaleUpperCase(), "\u0046\u004C", "LATIN SMALL LIGATURE FL");
assert.sameValue("\uFB03".toLocaleUpperCase(), "\u0046\u0046\u0049", "LATIN SMALL LIGATURE FFI");
assert.sameValue("\uFB04".toLocaleUpperCase(), "\u0046\u0046\u004C", "LATIN SMALL LIGATURE FFL");
assert.sameValue("\uFB05".toLocaleUpperCase(), "\u0053\u0054", "LATIN SMALL LIGATURE LONG S T");
assert.sameValue("\uFB06".toLocaleUpperCase(), "\u0053\u0054", "LATIN SMALL LIGATURE ST");

assert.sameValue("\u0587".toLocaleUpperCase(), "\u0535\u0552", "ARMENIAN SMALL LIGATURE ECH YIWN");
assert.sameValue("\uFB13".toLocaleUpperCase(), "\u0544\u0546", "ARMENIAN SMALL LIGATURE MEN NOW");
assert.sameValue("\uFB14".toLocaleUpperCase(), "\u0544\u0535", "ARMENIAN SMALL LIGATURE MEN ECH");
assert.sameValue("\uFB15".toLocaleUpperCase(), "\u0544\u053B", "ARMENIAN SMALL LIGATURE MEN INI");
assert.sameValue("\uFB16".toLocaleUpperCase(), "\u054E\u0546", "ARMENIAN SMALL LIGATURE VEW NOW");
assert.sameValue("\uFB17".toLocaleUpperCase(), "\u0544\u053D", "ARMENIAN SMALL LIGATURE MEN XEH");

assert.sameValue("\u0149".toLocaleUpperCase(), "\u02BC\u004E", "LATIN SMALL LETTER N PRECEDED BY APOSTROPHE");

assert.sameValue("\u0390".toLocaleUpperCase(), "\u0399\u0308\u0301", "GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS");
assert.sameValue("\u03B0".toLocaleUpperCase(), "\u03A5\u0308\u0301", "GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND TONOS");

assert.sameValue("\u01F0".toLocaleUpperCase(), "\u004A\u030C", "LATIN SMALL LETTER J WITH CARON");
assert.sameValue("\u1E96".toLocaleUpperCase(), "\u0048\u0331", "LATIN SMALL LETTER H WITH LINE BELOW");
assert.sameValue("\u1E97".toLocaleUpperCase(), "\u0054\u0308", "LATIN SMALL LETTER T WITH DIAERESIS");
assert.sameValue("\u1E98".toLocaleUpperCase(), "\u0057\u030A", "LATIN SMALL LETTER W WITH RING ABOVE");
assert.sameValue("\u1E99".toLocaleUpperCase(), "\u0059\u030A", "LATIN SMALL LETTER Y WITH RING ABOVE");
assert.sameValue("\u1E9A".toLocaleUpperCase(), "\u0041\u02BE", "LATIN SMALL LETTER A WITH RIGHT HALF RING");

assert.sameValue("\u1F50".toLocaleUpperCase(), "\u03A5\u0313", "GREEK SMALL LETTER UPSILON WITH PSILI");
assert.sameValue("\u1F52".toLocaleUpperCase(), "\u03A5\u0313\u0300", "GREEK SMALL LETTER UPSILON WITH PSILI AND VARIA");
assert.sameValue("\u1F54".toLocaleUpperCase(), "\u03A5\u0313\u0301", "GREEK SMALL LETTER UPSILON WITH PSILI AND OXIA");
assert.sameValue("\u1F56".toLocaleUpperCase(), "\u03A5\u0313\u0342", "GREEK SMALL LETTER UPSILON WITH PSILI AND PERISPOMENI");
assert.sameValue("\u1FB6".toLocaleUpperCase(), "\u0391\u0342", "GREEK SMALL LETTER ALPHA WITH PERISPOMENI");
assert.sameValue("\u1FC6".toLocaleUpperCase(), "\u0397\u0342", "GREEK SMALL LETTER ETA WITH PERISPOMENI");
assert.sameValue("\u1FD2".toLocaleUpperCase(), "\u0399\u0308\u0300", "GREEK SMALL LETTER IOTA WITH DIALYTIKA AND VARIA");
assert.sameValue("\u1FD3".toLocaleUpperCase(), "\u0399\u0308\u0301", "GREEK SMALL LETTER IOTA WITH DIALYTIKA AND OXIA");
assert.sameValue("\u1FD6".toLocaleUpperCase(), "\u0399\u0342", "GREEK SMALL LETTER IOTA WITH PERISPOMENI");
assert.sameValue("\u1FD7".toLocaleUpperCase(), "\u0399\u0308\u0342", "GREEK SMALL LETTER IOTA WITH DIALYTIKA AND PERISPOMENI");
assert.sameValue("\u1FE2".toLocaleUpperCase(), "\u03A5\u0308\u0300", "GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND VARIA");
assert.sameValue("\u1FE3".toLocaleUpperCase(), "\u03A5\u0308\u0301", "GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND OXIA");
assert.sameValue("\u1FE4".toLocaleUpperCase(), "\u03A1\u0313", "GREEK SMALL LETTER RHO WITH PSILI");
assert.sameValue("\u1FE6".toLocaleUpperCase(), "\u03A5\u0342", "GREEK SMALL LETTER UPSILON WITH PERISPOMENI");
assert.sameValue("\u1FE7".toLocaleUpperCase(), "\u03A5\u0308\u0342", "GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND PERISPOMENI");
assert.sameValue("\u1FF6".toLocaleUpperCase(), "\u03A9\u0342", "GREEK SMALL LETTER OMEGA WITH PERISPOMENI");

assert.sameValue("\u1F80".toLocaleUpperCase(), "\u1F08\u0399", "GREEK SMALL LETTER ALPHA WITH PSILI AND YPOGEGRAMMENI");
assert.sameValue("\u1F81".toLocaleUpperCase(), "\u1F09\u0399", "GREEK SMALL LETTER ALPHA WITH DASIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F82".toLocaleUpperCase(), "\u1F0A\u0399", "GREEK SMALL LETTER ALPHA WITH PSILI AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F83".toLocaleUpperCase(), "\u1F0B\u0399", "GREEK SMALL LETTER ALPHA WITH DASIA AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F84".toLocaleUpperCase(), "\u1F0C\u0399", "GREEK SMALL LETTER ALPHA WITH PSILI AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F85".toLocaleUpperCase(), "\u1F0D\u0399", "GREEK SMALL LETTER ALPHA WITH DASIA AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F86".toLocaleUpperCase(), "\u1F0E\u0399", "GREEK SMALL LETTER ALPHA WITH PSILI AND PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1F87".toLocaleUpperCase(), "\u1F0F\u0399", "GREEK SMALL LETTER ALPHA WITH DASIA AND PERISPOMENI AND YPOGEGRAMMENI");

assert.sameValue("\u1F88".toLocaleUpperCase(), "\u1F08\u0399", "GREEK CAPITAL LETTER ALPHA WITH PSILI AND PROSGEGRAMMENI");
assert.sameValue("\u1F89".toLocaleUpperCase(), "\u1F09\u0399", "GREEK CAPITAL LETTER ALPHA WITH DASIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8A".toLocaleUpperCase(), "\u1F0A\u0399", "GREEK CAPITAL LETTER ALPHA WITH PSILI AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8B".toLocaleUpperCase(), "\u1F0B\u0399", "GREEK CAPITAL LETTER ALPHA WITH DASIA AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8C".toLocaleUpperCase(), "\u1F0C\u0399", "GREEK CAPITAL LETTER ALPHA WITH PSILI AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8D".toLocaleUpperCase(), "\u1F0D\u0399", "GREEK CAPITAL LETTER ALPHA WITH DASIA AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8E".toLocaleUpperCase(), "\u1F0E\u0399", "GREEK CAPITAL LETTER ALPHA WITH PSILI AND PERISPOMENI AND PROSGEGRAMMENI");
assert.sameValue("\u1F8F".toLocaleUpperCase(), "\u1F0F\u0399", "GREEK CAPITAL LETTER ALPHA WITH DASIA AND PERISPOMENI AND PROSGEGRAMMENI");

assert.sameValue("\u1F90".toLocaleUpperCase(), "\u1F28\u0399", "GREEK SMALL LETTER ETA WITH PSILI AND YPOGEGRAMMENI");
assert.sameValue("\u1F91".toLocaleUpperCase(), "\u1F29\u0399", "GREEK SMALL LETTER ETA WITH DASIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F92".toLocaleUpperCase(), "\u1F2A\u0399", "GREEK SMALL LETTER ETA WITH PSILI AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F93".toLocaleUpperCase(), "\u1F2B\u0399", "GREEK SMALL LETTER ETA WITH DASIA AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F94".toLocaleUpperCase(), "\u1F2C\u0399", "GREEK SMALL LETTER ETA WITH PSILI AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F95".toLocaleUpperCase(), "\u1F2D\u0399", "GREEK SMALL LETTER ETA WITH DASIA AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F96".toLocaleUpperCase(), "\u1F2E\u0399", "GREEK SMALL LETTER ETA WITH PSILI AND PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1F97".toLocaleUpperCase(), "\u1F2F\u0399", "GREEK SMALL LETTER ETA WITH DASIA AND PERISPOMENI AND YPOGEGRAMMENI");

assert.sameValue("\u1F98".toLocaleUpperCase(), "\u1F28\u0399", "GREEK CAPITAL LETTER ETA WITH PSILI AND PROSGEGRAMMENI");
assert.sameValue("\u1F99".toLocaleUpperCase(), "\u1F29\u0399", "GREEK CAPITAL LETTER ETA WITH DASIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9A".toLocaleUpperCase(), "\u1F2A\u0399", "GREEK CAPITAL LETTER ETA WITH PSILI AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9B".toLocaleUpperCase(), "\u1F2B\u0399", "GREEK CAPITAL LETTER ETA WITH DASIA AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9C".toLocaleUpperCase(), "\u1F2C\u0399", "GREEK CAPITAL LETTER ETA WITH PSILI AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9D".toLocaleUpperCase(), "\u1F2D\u0399", "GREEK CAPITAL LETTER ETA WITH DASIA AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9E".toLocaleUpperCase(), "\u1F2E\u0399", "GREEK CAPITAL LETTER ETA WITH PSILI AND PERISPOMENI AND PROSGEGRAMMENI");
assert.sameValue("\u1F9F".toLocaleUpperCase(), "\u1F2F\u0399", "GREEK CAPITAL LETTER ETA WITH DASIA AND PERISPOMENI AND PROSGEGRAMMENI");

assert.sameValue("\u1FA0".toLocaleUpperCase(), "\u1F68\u0399", "GREEK SMALL LETTER OMEGA WITH PSILI AND YPOGEGRAMMENI");
assert.sameValue("\u1FA1".toLocaleUpperCase(), "\u1F69\u0399", "GREEK SMALL LETTER OMEGA WITH DASIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA2".toLocaleUpperCase(), "\u1F6A\u0399", "GREEK SMALL LETTER OMEGA WITH PSILI AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA3".toLocaleUpperCase(), "\u1F6B\u0399", "GREEK SMALL LETTER OMEGA WITH DASIA AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA4".toLocaleUpperCase(), "\u1F6C\u0399", "GREEK SMALL LETTER OMEGA WITH PSILI AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA5".toLocaleUpperCase(), "\u1F6D\u0399", "GREEK SMALL LETTER OMEGA WITH DASIA AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA6".toLocaleUpperCase(), "\u1F6E\u0399", "GREEK SMALL LETTER OMEGA WITH PSILI AND PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1FA7".toLocaleUpperCase(), "\u1F6F\u0399", "GREEK SMALL LETTER OMEGA WITH DASIA AND PERISPOMENI AND YPOGEGRAMMENI");

assert.sameValue("\u1FA8".toLocaleUpperCase(), "\u1F68\u0399", "GREEK CAPITAL LETTER OMEGA WITH PSILI AND PROSGEGRAMMENI");
assert.sameValue("\u1FA9".toLocaleUpperCase(), "\u1F69\u0399", "GREEK CAPITAL LETTER OMEGA WITH DASIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAA".toLocaleUpperCase(), "\u1F6A\u0399", "GREEK CAPITAL LETTER OMEGA WITH PSILI AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAB".toLocaleUpperCase(), "\u1F6B\u0399", "GREEK CAPITAL LETTER OMEGA WITH DASIA AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAC".toLocaleUpperCase(), "\u1F6C\u0399", "GREEK CAPITAL LETTER OMEGA WITH PSILI AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAD".toLocaleUpperCase(), "\u1F6D\u0399", "GREEK CAPITAL LETTER OMEGA WITH DASIA AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAE".toLocaleUpperCase(), "\u1F6E\u0399", "GREEK CAPITAL LETTER OMEGA WITH PSILI AND PERISPOMENI AND PROSGEGRAMMENI");
assert.sameValue("\u1FAF".toLocaleUpperCase(), "\u1F6F\u0399", "GREEK CAPITAL LETTER OMEGA WITH DASIA AND PERISPOMENI AND PROSGEGRAMMENI");

assert.sameValue("\u1FB3".toLocaleUpperCase(), "\u0391\u0399", "GREEK SMALL LETTER ALPHA WITH YPOGEGRAMMENI");
assert.sameValue("\u1FBC".toLocaleUpperCase(), "\u0391\u0399", "GREEK CAPITAL LETTER ALPHA WITH PROSGEGRAMMENI");
assert.sameValue("\u1FC3".toLocaleUpperCase(), "\u0397\u0399", "GREEK SMALL LETTER ETA WITH YPOGEGRAMMENI");
assert.sameValue("\u1FCC".toLocaleUpperCase(), "\u0397\u0399", "GREEK CAPITAL LETTER ETA WITH PROSGEGRAMMENI");
assert.sameValue("\u1FF3".toLocaleUpperCase(), "\u03A9\u0399", "GREEK SMALL LETTER OMEGA WITH YPOGEGRAMMENI");
assert.sameValue("\u1FFC".toLocaleUpperCase(), "\u03A9\u0399", "GREEK CAPITAL LETTER OMEGA WITH PROSGEGRAMMENI");

assert.sameValue("\u1FB2".toLocaleUpperCase(), "\u1FBA\u0399", "GREEK SMALL LETTER ALPHA WITH VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FB4".toLocaleUpperCase(), "\u0386\u0399", "GREEK SMALL LETTER ALPHA WITH OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FC2".toLocaleUpperCase(), "\u1FCA\u0399", "GREEK SMALL LETTER ETA WITH VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FC4".toLocaleUpperCase(), "\u0389\u0399", "GREEK SMALL LETTER ETA WITH OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FF2".toLocaleUpperCase(), "\u1FFA\u0399", "GREEK SMALL LETTER OMEGA WITH VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FF4".toLocaleUpperCase(), "\u038F\u0399", "GREEK SMALL LETTER OMEGA WITH OXIA AND YPOGEGRAMMENI");

assert.sameValue("\u1FB7".toLocaleUpperCase(), "\u0391\u0342\u0399", "GREEK SMALL LETTER ALPHA WITH PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1FC7".toLocaleUpperCase(), "\u0397\u0342\u0399", "GREEK SMALL LETTER ETA WITH PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1FF7".toLocaleUpperCase(), "\u03A9\u0342\u0399", "GREEK SMALL LETTER OMEGA WITH PERISPOMENI AND YPOGEGRAMMENI");
