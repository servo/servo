// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: URI tests
esid: sec-decodeuricomponent-encodeduricomponent
description: Checking RUSSIAN ALPHABET
---*/

//CHECK#1
if (decodeURIComponent("http://ru.wikipedia.org/wiki/%d0%ae%D0%bd%D0%B8%D0%BA%D0%BE%D0%B4") !== "http://ru.wikipedia.org/wiki/Юникод") {
  throw new Test262Error('#1: http://ru.wikipedia.org/wiki/Юникод');
}

//CHECK#2
if (decodeURIComponent("http://ru.wikipedia.org/wiki/%D0%AE%D0%BD%D0%B8%D0%BA%D0%BE%D0%B4#%D0%A1%D1%81%D1%8B%D0%BB%D0%BA%D0%B8") !== "http://ru.wikipedia.org/wiki/Юникод#Ссылки") {
  throw new Test262Error('#2: http://ru.wikipedia.org/wiki/Юникод#Ссылки');
}

//CHECK#3
if (decodeURIComponent("http://ru.wikipedia.org/wiki/%D0%AE%D0%BD%D0%B8%D0%BA%D0%BE%D0%B4%23%D0%92%D0%B5%D1%80%D1%81%D0%B8%D0%B8%20%D0%AE%D0%BD%D0%B8%D0%BA%D0%BE%D0%B4%D0%B0") !== "http://ru.wikipedia.org/wiki/Юникод#Версии Юникода") {
  throw new Test262Error('#3: http://ru.wikipedia.org/wiki/Юникод%23Версии Юникода');
}
