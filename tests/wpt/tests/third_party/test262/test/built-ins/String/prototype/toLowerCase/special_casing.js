// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Check if String.prototype.toLowerCase supports mappings defined in SpecialCasings
info: |
    The result must be derived according to the locale-insensitive case mappings in the Unicode Character
    Database (this explicitly includes not only the UnicodeData.txt file, but also all locale-insensitive
    mappings in the SpecialCasings.txt file that accompanies it).
es5id: 15.5.4.16
es6id: 21.1.3.22
---*/

// SpecialCasing.txt, except for conditional mappings.

assert.sameValue("\u00DF".toLowerCase(), "\u00DF", "LATIN SMALL LETTER SHARP S");

assert.sameValue("\u0130".toLowerCase(), "\u0069\u0307", "LATIN CAPITAL LETTER I WITH DOT ABOVE");

assert.sameValue("\uFB00".toLowerCase(), "\uFB00", "LATIN SMALL LIGATURE FF");
assert.sameValue("\uFB01".toLowerCase(), "\uFB01", "LATIN SMALL LIGATURE FI");
assert.sameValue("\uFB02".toLowerCase(), "\uFB02", "LATIN SMALL LIGATURE FL");
assert.sameValue("\uFB03".toLowerCase(), "\uFB03", "LATIN SMALL LIGATURE FFI");
assert.sameValue("\uFB04".toLowerCase(), "\uFB04", "LATIN SMALL LIGATURE FFL");
assert.sameValue("\uFB05".toLowerCase(), "\uFB05", "LATIN SMALL LIGATURE LONG S T");
assert.sameValue("\uFB06".toLowerCase(), "\uFB06", "LATIN SMALL LIGATURE ST");

assert.sameValue("\u0587".toLowerCase(), "\u0587", "ARMENIAN SMALL LIGATURE ECH YIWN");
assert.sameValue("\uFB13".toLowerCase(), "\uFB13", "ARMENIAN SMALL LIGATURE MEN NOW");
assert.sameValue("\uFB14".toLowerCase(), "\uFB14", "ARMENIAN SMALL LIGATURE MEN ECH");
assert.sameValue("\uFB15".toLowerCase(), "\uFB15", "ARMENIAN SMALL LIGATURE MEN INI");
assert.sameValue("\uFB16".toLowerCase(), "\uFB16", "ARMENIAN SMALL LIGATURE VEW NOW");
assert.sameValue("\uFB17".toLowerCase(), "\uFB17", "ARMENIAN SMALL LIGATURE MEN XEH");

assert.sameValue("\u0149".toLowerCase(), "\u0149", "LATIN SMALL LETTER N PRECEDED BY APOSTROPHE");

assert.sameValue("\u0390".toLowerCase(), "\u0390", "GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS");
assert.sameValue("\u03B0".toLowerCase(), "\u03B0", "GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND TONOS");

assert.sameValue("\u01F0".toLowerCase(), "\u01F0", "LATIN SMALL LETTER J WITH CARON");
assert.sameValue("\u1E96".toLowerCase(), "\u1E96", "LATIN SMALL LETTER H WITH LINE BELOW");
assert.sameValue("\u1E97".toLowerCase(), "\u1E97", "LATIN SMALL LETTER T WITH DIAERESIS");
assert.sameValue("\u1E98".toLowerCase(), "\u1E98", "LATIN SMALL LETTER W WITH RING ABOVE");
assert.sameValue("\u1E99".toLowerCase(), "\u1E99", "LATIN SMALL LETTER Y WITH RING ABOVE");
assert.sameValue("\u1E9A".toLowerCase(), "\u1E9A", "LATIN SMALL LETTER A WITH RIGHT HALF RING");

assert.sameValue("\u1F50".toLowerCase(), "\u1F50", "GREEK SMALL LETTER UPSILON WITH PSILI");
assert.sameValue("\u1F52".toLowerCase(), "\u1F52", "GREEK SMALL LETTER UPSILON WITH PSILI AND VARIA");
assert.sameValue("\u1F54".toLowerCase(), "\u1F54", "GREEK SMALL LETTER UPSILON WITH PSILI AND OXIA");
assert.sameValue("\u1F56".toLowerCase(), "\u1F56", "GREEK SMALL LETTER UPSILON WITH PSILI AND PERISPOMENI");
assert.sameValue("\u1FB6".toLowerCase(), "\u1FB6", "GREEK SMALL LETTER ALPHA WITH PERISPOMENI");
assert.sameValue("\u1FC6".toLowerCase(), "\u1FC6", "GREEK SMALL LETTER ETA WITH PERISPOMENI");
assert.sameValue("\u1FD2".toLowerCase(), "\u1FD2", "GREEK SMALL LETTER IOTA WITH DIALYTIKA AND VARIA");
assert.sameValue("\u1FD3".toLowerCase(), "\u1FD3", "GREEK SMALL LETTER IOTA WITH DIALYTIKA AND OXIA");
assert.sameValue("\u1FD6".toLowerCase(), "\u1FD6", "GREEK SMALL LETTER IOTA WITH PERISPOMENI");
assert.sameValue("\u1FD7".toLowerCase(), "\u1FD7", "GREEK SMALL LETTER IOTA WITH DIALYTIKA AND PERISPOMENI");
assert.sameValue("\u1FE2".toLowerCase(), "\u1FE2", "GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND VARIA");
assert.sameValue("\u1FE3".toLowerCase(), "\u1FE3", "GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND OXIA");
assert.sameValue("\u1FE4".toLowerCase(), "\u1FE4", "GREEK SMALL LETTER RHO WITH PSILI");
assert.sameValue("\u1FE6".toLowerCase(), "\u1FE6", "GREEK SMALL LETTER UPSILON WITH PERISPOMENI");
assert.sameValue("\u1FE7".toLowerCase(), "\u1FE7", "GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND PERISPOMENI");
assert.sameValue("\u1FF6".toLowerCase(), "\u1FF6", "GREEK SMALL LETTER OMEGA WITH PERISPOMENI");

assert.sameValue("\u1F80".toLowerCase(), "\u1F80", "GREEK SMALL LETTER ALPHA WITH PSILI AND YPOGEGRAMMENI");
assert.sameValue("\u1F81".toLowerCase(), "\u1F81", "GREEK SMALL LETTER ALPHA WITH DASIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F82".toLowerCase(), "\u1F82", "GREEK SMALL LETTER ALPHA WITH PSILI AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F83".toLowerCase(), "\u1F83", "GREEK SMALL LETTER ALPHA WITH DASIA AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F84".toLowerCase(), "\u1F84", "GREEK SMALL LETTER ALPHA WITH PSILI AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F85".toLowerCase(), "\u1F85", "GREEK SMALL LETTER ALPHA WITH DASIA AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F86".toLowerCase(), "\u1F86", "GREEK SMALL LETTER ALPHA WITH PSILI AND PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1F87".toLowerCase(), "\u1F87", "GREEK SMALL LETTER ALPHA WITH DASIA AND PERISPOMENI AND YPOGEGRAMMENI");

assert.sameValue("\u1F88".toLowerCase(), "\u1F80", "GREEK CAPITAL LETTER ALPHA WITH PSILI AND PROSGEGRAMMENI");
assert.sameValue("\u1F89".toLowerCase(), "\u1F81", "GREEK CAPITAL LETTER ALPHA WITH DASIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8A".toLowerCase(), "\u1F82", "GREEK CAPITAL LETTER ALPHA WITH PSILI AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8B".toLowerCase(), "\u1F83", "GREEK CAPITAL LETTER ALPHA WITH DASIA AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8C".toLowerCase(), "\u1F84", "GREEK CAPITAL LETTER ALPHA WITH PSILI AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8D".toLowerCase(), "\u1F85", "GREEK CAPITAL LETTER ALPHA WITH DASIA AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F8E".toLowerCase(), "\u1F86", "GREEK CAPITAL LETTER ALPHA WITH PSILI AND PERISPOMENI AND PROSGEGRAMMENI");
assert.sameValue("\u1F8F".toLowerCase(), "\u1F87", "GREEK CAPITAL LETTER ALPHA WITH DASIA AND PERISPOMENI AND PROSGEGRAMMENI");

assert.sameValue("\u1F90".toLowerCase(), "\u1F90", "GREEK SMALL LETTER ETA WITH PSILI AND YPOGEGRAMMENI");
assert.sameValue("\u1F91".toLowerCase(), "\u1F91", "GREEK SMALL LETTER ETA WITH DASIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F92".toLowerCase(), "\u1F92", "GREEK SMALL LETTER ETA WITH PSILI AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F93".toLowerCase(), "\u1F93", "GREEK SMALL LETTER ETA WITH DASIA AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F94".toLowerCase(), "\u1F94", "GREEK SMALL LETTER ETA WITH PSILI AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F95".toLowerCase(), "\u1F95", "GREEK SMALL LETTER ETA WITH DASIA AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1F96".toLowerCase(), "\u1F96", "GREEK SMALL LETTER ETA WITH PSILI AND PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1F97".toLowerCase(), "\u1F97", "GREEK SMALL LETTER ETA WITH DASIA AND PERISPOMENI AND YPOGEGRAMMENI");

assert.sameValue("\u1F98".toLowerCase(), "\u1F90", "GREEK CAPITAL LETTER ETA WITH PSILI AND PROSGEGRAMMENI");
assert.sameValue("\u1F99".toLowerCase(), "\u1F91", "GREEK CAPITAL LETTER ETA WITH DASIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9A".toLowerCase(), "\u1F92", "GREEK CAPITAL LETTER ETA WITH PSILI AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9B".toLowerCase(), "\u1F93", "GREEK CAPITAL LETTER ETA WITH DASIA AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9C".toLowerCase(), "\u1F94", "GREEK CAPITAL LETTER ETA WITH PSILI AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9D".toLowerCase(), "\u1F95", "GREEK CAPITAL LETTER ETA WITH DASIA AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1F9E".toLowerCase(), "\u1F96", "GREEK CAPITAL LETTER ETA WITH PSILI AND PERISPOMENI AND PROSGEGRAMMENI");
assert.sameValue("\u1F9F".toLowerCase(), "\u1F97", "GREEK CAPITAL LETTER ETA WITH DASIA AND PERISPOMENI AND PROSGEGRAMMENI");

assert.sameValue("\u1FA0".toLowerCase(), "\u1FA0", "GREEK SMALL LETTER OMEGA WITH PSILI AND YPOGEGRAMMENI");
assert.sameValue("\u1FA1".toLowerCase(), "\u1FA1", "GREEK SMALL LETTER OMEGA WITH DASIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA2".toLowerCase(), "\u1FA2", "GREEK SMALL LETTER OMEGA WITH PSILI AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA3".toLowerCase(), "\u1FA3", "GREEK SMALL LETTER OMEGA WITH DASIA AND VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA4".toLowerCase(), "\u1FA4", "GREEK SMALL LETTER OMEGA WITH PSILI AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA5".toLowerCase(), "\u1FA5", "GREEK SMALL LETTER OMEGA WITH DASIA AND OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FA6".toLowerCase(), "\u1FA6", "GREEK SMALL LETTER OMEGA WITH PSILI AND PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1FA7".toLowerCase(), "\u1FA7", "GREEK SMALL LETTER OMEGA WITH DASIA AND PERISPOMENI AND YPOGEGRAMMENI");

assert.sameValue("\u1FA8".toLowerCase(), "\u1FA0", "GREEK CAPITAL LETTER OMEGA WITH PSILI AND PROSGEGRAMMENI");
assert.sameValue("\u1FA9".toLowerCase(), "\u1FA1", "GREEK CAPITAL LETTER OMEGA WITH DASIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAA".toLowerCase(), "\u1FA2", "GREEK CAPITAL LETTER OMEGA WITH PSILI AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAB".toLowerCase(), "\u1FA3", "GREEK CAPITAL LETTER OMEGA WITH DASIA AND VARIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAC".toLowerCase(), "\u1FA4", "GREEK CAPITAL LETTER OMEGA WITH PSILI AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAD".toLowerCase(), "\u1FA5", "GREEK CAPITAL LETTER OMEGA WITH DASIA AND OXIA AND PROSGEGRAMMENI");
assert.sameValue("\u1FAE".toLowerCase(), "\u1FA6", "GREEK CAPITAL LETTER OMEGA WITH PSILI AND PERISPOMENI AND PROSGEGRAMMENI");
assert.sameValue("\u1FAF".toLowerCase(), "\u1FA7", "GREEK CAPITAL LETTER OMEGA WITH DASIA AND PERISPOMENI AND PROSGEGRAMMENI");

assert.sameValue("\u1FB3".toLowerCase(), "\u1FB3", "GREEK SMALL LETTER ALPHA WITH YPOGEGRAMMENI");
assert.sameValue("\u1FBC".toLowerCase(), "\u1FB3", "GREEK CAPITAL LETTER ALPHA WITH PROSGEGRAMMENI");
assert.sameValue("\u1FC3".toLowerCase(), "\u1FC3", "GREEK SMALL LETTER ETA WITH YPOGEGRAMMENI");
assert.sameValue("\u1FCC".toLowerCase(), "\u1FC3", "GREEK CAPITAL LETTER ETA WITH PROSGEGRAMMENI");
assert.sameValue("\u1FF3".toLowerCase(), "\u1FF3", "GREEK SMALL LETTER OMEGA WITH YPOGEGRAMMENI");
assert.sameValue("\u1FFC".toLowerCase(), "\u1FF3", "GREEK CAPITAL LETTER OMEGA WITH PROSGEGRAMMENI");

assert.sameValue("\u1FB2".toLowerCase(), "\u1FB2", "GREEK SMALL LETTER ALPHA WITH VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FB4".toLowerCase(), "\u1FB4", "GREEK SMALL LETTER ALPHA WITH OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FC2".toLowerCase(), "\u1FC2", "GREEK SMALL LETTER ETA WITH VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FC4".toLowerCase(), "\u1FC4", "GREEK SMALL LETTER ETA WITH OXIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FF2".toLowerCase(), "\u1FF2", "GREEK SMALL LETTER OMEGA WITH VARIA AND YPOGEGRAMMENI");
assert.sameValue("\u1FF4".toLowerCase(), "\u1FF4", "GREEK SMALL LETTER OMEGA WITH OXIA AND YPOGEGRAMMENI");

assert.sameValue("\u1FB7".toLowerCase(), "\u1FB7", "GREEK SMALL LETTER ALPHA WITH PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1FC7".toLowerCase(), "\u1FC7", "GREEK SMALL LETTER ETA WITH PERISPOMENI AND YPOGEGRAMMENI");
assert.sameValue("\u1FF7".toLowerCase(), "\u1FF7", "GREEK SMALL LETTER OMEGA WITH PERISPOMENI AND YPOGEGRAMMENI");
