// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: unescapedURIComponentSet not containing "#"
esid: sec-encodeuricomponent-uricomponent
description: encodeURIComponent("#") === "%23"
---*/

if (encodeURIComponent("#") !== "%23") {
  throw new Test262Error('#1: unescapedURIComponentSet not containing "%23"');
}
