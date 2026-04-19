// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: URI tests
esid: sec-encodeuri-uri
description: Checking ENGLISH ALPHABET
---*/

//CHECK#1
if (encodeURI("http://unipro.ru/0123456789") !== "http://unipro.ru/0123456789") {
  throw new Test262Error('#1: http://unipro.ru/0123456789');
}

//CHECK#2
if (encodeURI("aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ") !== "aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ") {
  throw new Test262Error('#2: aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ');
}

//CHECK#3
if (encodeURI("aA_bB-cC.dD!eE~fF*gG'hH(iI)jJ;kK/lL?mM:nN@oO&pP=qQ+rR$sS,tT9uU8vV7wW6xX5yY4zZ") !== "aA_bB-cC.dD!eE~fF*gG'hH(iI)jJ;kK/lL?mM:nN@oO&pP=qQ+rR$sS,tT9uU8vV7wW6xX5yY4zZ") {
  throw new Test262Error('#3: ');
}
