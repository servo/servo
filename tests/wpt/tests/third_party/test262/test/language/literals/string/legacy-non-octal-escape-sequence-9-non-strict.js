// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-literals-string-literals
description: LegacyOctalEscapeSequence is not available in non-strict code - 9
info: |
    EscapeSequence ::
      CharacterEscapeSequence
      LegacyOctalEscapeSequence
      NonOctalDecimalEscapeSequence
      HexEscapeSequence
      UnicodeEscapeSequence

    NonOctalDecimalEscapeSequence :: one of
      8 9
flags: [noStrict]
---*/

assert.sameValue('\9', '9');
