// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.12
description: >
  Returns normalized string from coerced form.
info: |
  21.1.3.12 String.prototype.normalize ( [ form ] )

  ...
  4. If form is not provided or form is undefined, let form be "NFC".
  5. Let f be ToString(form).
  6. ReturnIfAbrupt(f).
  7. If f is not one of "NFC", "NFD", "NFKC", or "NFKD", throw a RangeError
  exception.
  8. Let ns be the String value that is the result of normalizing S into the
  normalization form named by f as specified in
  http://www.unicode.org/reports/tr15/tr15-29.html.
  9. Return ns.
---*/

var s = '\u00C5\u2ADC\u0958\u2126\u0344';
var nfc = '\xC5\u2ADD\u0338\u0915\u093C\u03A9\u0308\u0301';
var nfd = 'A\u030A\u2ADD\u0338\u0915\u093C\u03A9\u0308\u0301';
var o = {
  toString: function() {
    return 'NFC';
  }
};

assert.sameValue(s.normalize(['NFC']), nfc, 'coerced array - NFC');
assert.sameValue(s.normalize(o), nfc, 'coerced object - NFC');

o = {
  toString: function() {
    return 'NFD';
  }
};

assert.sameValue(s.normalize(['NFD']), nfd, 'coerced array - NFD');
assert.sameValue(s.normalize(o), nfd, 'coerced object - NFD');
