// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.2
description: >
  The the arguments list is empty, an empty string is returned.
info: |
  String.fromCodePoint ( ...codePoints )

  1. Let result be the empty String.
  2. For each element next of codePoints, do
    ...
  3. Assert: If codePoints is empty, then result is the empty String.
  4. Return result.
features: [String.fromCodePoint]
---*/

assert.sameValue(String.fromCodePoint(), '');
