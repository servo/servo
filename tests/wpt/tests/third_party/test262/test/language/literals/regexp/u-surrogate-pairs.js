// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Out-of-range value of hexadecimal digits in UnicodeEscapeSequence
es6id: 21.2.2.10
info: |
    21.2.2.10 CharacterEscape

    The production RegExpUnicodeEscapeSequence :: u LeadSurrogate \u
    TrailSurrogate evaluates as follows:

        1. Let lead be the result of evaluating LeadSurrogate.
        2. Let trail be the result of evaluating TrailSurrogate.
        3. Let cp be UTF16Decode(lead, trail).
        4. Return the character whose character value is cp.
---*/

assert(/^[\ud834\udf06]$/u.test('\ud834\udf06'));
