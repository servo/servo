// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    parseInt is no longer allowed to treat a leading zero as indicating
    octal.  "If radix is undefined or 0, it is assumed to be 10 except
    when the number begins with the character pairs 0x or 0X, in which
    case a radix of 16 is assumed."
esid: sec-parseint-string-radix
description: Check if parseInt still accepts octal
---*/

assert.sameValue(parseInt('010'), 10, 'parseInt(\'010\') must return 10');
