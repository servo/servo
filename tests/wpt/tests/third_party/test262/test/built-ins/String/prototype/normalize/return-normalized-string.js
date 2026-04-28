// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.12
description: >
  Returns normalized string.
info: |
  21.1.3.12 String.prototype.normalize ( [ form ] )

  ...
  7. If f is not one of "NFC", "NFD", "NFKC", or "NFKD", throw a RangeError
  exception.
  8. Let ns be the String value that is the result of normalizing S into the
  normalization form named by f as specified in
  http://www.unicode.org/reports/tr15/tr15-29.html.
  9. Return ns.
---*/

var s = '\u1E9B\u0323';

assert.sameValue(s.normalize('NFC'), '\u1E9B\u0323', 'Normalized on NFC');
assert.sameValue(s.normalize('NFD'), '\u017F\u0323\u0307', 'Normalized on NFD');
assert.sameValue(s.normalize('NFKC'), '\u1E69', 'Normalized on NFKC');
assert.sameValue(s.normalize('NFKD'), '\u0073\u0323\u0307', 'Normalized on NFKD');

assert.sameValue(
  '\u00C5\u2ADC\u0958\u2126\u0344'.normalize('NFC'),
  '\xC5\u2ADD\u0338\u0915\u093C\u03A9\u0308\u0301',
  'Normalized on NFC'
);

assert.sameValue(
  '\u00C5\u2ADC\u0958\u2126\u0344'.normalize('NFD'),
  'A\u030A\u2ADD\u0338\u0915\u093C\u03A9\u0308\u0301',
  'Normalized on NFD'
);

assert.sameValue(
  '\u00C5\u2ADC\u0958\u2126\u0344'.normalize('NFKC'),
  '\xC5\u2ADD\u0338\u0915\u093C\u03A9\u0308\u0301',
  'Normalized on NFKC'
);

assert.sameValue(
  '\u00C5\u2ADC\u0958\u2126\u0344'.normalize('NFKD'),
  'A\u030A\u2ADD\u0338\u0915\u093C\u03A9\u0308\u0301',
  'Normalized on NFKD'
);
