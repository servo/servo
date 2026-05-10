// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 11.1.8.6.1
description: Template values of the zero width no-break space character
info: |
    The zero width no-break space format-control character may be used within
    template literals.
---*/

var callCount;

callCount = 0;
(function(s) {
  callCount++;
  assert.sameValue(
    s[0], '﻿test', 'TV (specified via unicode escape sequence)'
  );
  assert.sameValue(
    s.raw[0], '\\uFEFFtest', 'TV (specified via unicode escape sequence)'
  );
})`\uFEFFtest`;
assert.sameValue(callCount, 1);

callCount = 0;
(function(s) {
  callCount++;
  assert.sameValue(s[0], '﻿test', 'TV (specified via literal character)');
  assert.sameValue(
    s.raw[0], '﻿test', 'TV (specified via literal character)'
  );
})`﻿test`;
assert.sameValue(callCount, 1);
