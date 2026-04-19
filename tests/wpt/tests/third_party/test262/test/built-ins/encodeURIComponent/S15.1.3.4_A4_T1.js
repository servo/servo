// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: URI tests
esid: sec-encodeuricomponent-uricomponent
description: Checking ENGLISH ALPHABET
---*/

//CHECK#1
if (encodeURIComponent("http://unipro.ru/0123456789") !== "http%3A%2F%2Funipro.ru%2F0123456789") {
  throw new Test262Error('#1: http://unipro.ru/0123456789');
}

//CHECK#2
if (encodeURIComponent("aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ") !== "aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ") {
  throw new Test262Error('#2: aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ');
}

//CHECK#3
if (encodeURIComponent(";/?:@&=+$,") !== "%3B%2F%3F%3A%40%26%3D%2B%24%2C") {
  throw new Test262Error('#3: ');
}
