// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-finalization-registry.prototype.register
description: Return undefined (applying custom this)
info: |
  FinalizationRegistry.prototype.register ( target , holdings [, unregisterToken ] )

  1. Let finalizationRegistry be the this value.
  2. If Type(finalizationRegistry) is not Object, throw a TypeError exception.
  3. If Type(target) is not Object, throw a TypeError exception.
  4. If finalizationRegistry does not have a [[Cells]] internal slot, throw a TypeError exception.
  5. If Type(unregisterToken) is not Object,
    a. If unregisterToken is not undefined, throw a TypeError exception.
    b. Set unregisterToken to empty.
  6. Let cell be the Record { [[Target]] : target, [[Holdings]]: holdings, [[UnregisterToken]]: unregisterToken }.
  7. Append cell to finalizationRegistry.[[Cells]].
  8. Return undefined.
features: [FinalizationRegistry]
---*/

var fn = function() {};
var register = FinalizationRegistry.prototype.register;
var finalizationRegistry = new FinalizationRegistry(fn);

var target = {};
assert.sameValue(register.call(finalizationRegistry, target), undefined);
