// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "The MV of StringNumericLiteral ::: [empty] is 0"
es5id: 9.3.1_A1
description: Number('') convert to Number by explicit transformation
---*/
assert.sameValue(Number(""), 0, 'Number("") must return 0');
