// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: unescapedURISet containing "#"
esid: sec-encodeuri-uri
description: encodeURI("#") === "#"
---*/

if (encodeURI("#") !== "#") {
  throw new Test262Error('#1: unescapedURISet containing "#"');
}
