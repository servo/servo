// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-escape-string
es6id: B.2.1.1
description: Escaping of code units below 255
info: |
    [...]
    5. Repeat, while k < length,
       a. Let char be the code unit (represented as a 16-bit unsigned integer)
          at index k within string.
       [...]
       d. Else char < 256,
          i. Let S be a String containing three code units "%xy" where xy are
             the code units of two uppercase hexadecimal digits encoding the
             value of char.
       [...]
---*/

assert.sameValue(
  escape('\x00\x01\x02\x03'),
  '%00%01%02%03',
  'characters: \\x00\\x01\\x02\\x03'
);

assert.sameValue(
  escape('!"#$%&\'()'),
  '%21%22%23%24%25%26%27%28%29',
  'characters preceding "*": !"#$%&\'()'
);

assert.sameValue(escape(','), '%2C', 'character between "+" and "-": ,');

assert.sameValue(
  escape(':;<=>?'),
  '%3A%3B%3C%3D%3E%3F',
  'characters between "9" and "@": :;<=>?'
);

assert.sameValue(
  escape('[\\]^'), '%5B%5C%5D%5E', 'characters between "Z" and "_": [\\]^'
);

assert.sameValue(escape('`'), '%60', 'character between "_" and "a": `');

assert.sameValue(
  escape('{|}~\x7f\x80'),
  '%7B%7C%7D%7E%7F%80',
  'characters following "z": {|}~\\x7f\\x80'
);

assert.sameValue(
  escape('\xfd\xfe\xff'), '%FD%FE%FF', '\\xfd\\xfe\\xff'
);
