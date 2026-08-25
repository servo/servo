/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var f = function(){};
for (var y in f);
f.j = 0;
Object.defineProperty(f, "k", ({configurable: true}));
delete f.j;
Object.defineProperty(f, "k", ({get: function() {}}));

