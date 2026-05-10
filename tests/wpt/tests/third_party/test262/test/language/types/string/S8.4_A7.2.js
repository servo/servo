// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-line-terminators
description: >
  Line terminator <CR> may appear as an escape sequence within a StringLiteral
info: |
  A line terminator cannot occur within any token except a StringLiteral, Template, or TemplateSubstitutionTail.
---*/

assert.throws(ReferenceError, function() {
  eval("var x = asdf\u000Dghjk");
});


