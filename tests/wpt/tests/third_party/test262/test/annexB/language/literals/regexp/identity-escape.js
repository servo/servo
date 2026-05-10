// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regular-expressions-patterns
es6id: B.1.4
description: Support for UnicodeIDContinue in IdentityEscape
info: |
    IdentityEscape[U] ::
        [+U] SyntaxCharacter
        [+U] /
        [~U] SourceCharacter but not c
---*/

var match;

match = /\C/.exec('ABCDE');
assert.sameValue(match[0], 'C');

match = /O\PQ/.exec('MNOPQRS');
assert.sameValue(match[0], 'OPQ');

match = /\8/.exec('789');
assert.sameValue(match[0], '8');

match = /7\89/.exec('67890');
assert.sameValue(match[0], '789');

match = /\9/.exec('890');
assert.sameValue(match[0], '9');

match = /8\90/.exec('78900');
assert.sameValue(match[0], '890');

match = /(.)(.)(.)(.)(.)(.)(.)(.)\8\8/.exec('0123456777');
assert.sameValue(
  match[0],
  '0123456777',
  'DecimalEscape takes precedence over IdentityEscape (\\8)'
);

match = /(.)(.)(.)(.)(.)(.)(.)(.)(.)\9\9/.exec('01234567888');
assert.sameValue(
  match[0],
  '01234567888',
  'DecimalEscape takes precedence over IdentityEscape (\\9)'
);
