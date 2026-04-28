// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of boolean conversion from nonempty string value (length is not
    zero) is true; from empty String (length is zero) is false
esid: sec-toboolean
description: Any nonempty string convert to Boolean by explicit transformation
---*/
assert.sameValue(Boolean(" "), true, 'Boolean(" ") must return true');
assert.sameValue(Boolean("Nonempty String"), true, 'Boolean("Nonempty String") must return true');
