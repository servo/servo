// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.10.4.1-3
description: >
    RegExp - the thrown error is SyntaxError instead of RegExpError
    when 'F' contains any character other than 'g', 'i', or 'm'
---*/

assert.throws(SyntaxError, function() {
  var regExpObj = new RegExp('abc', 'a');
});
