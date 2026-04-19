// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol.for
description: Creation of a unique Symbol value
info: |
    1. Let stringKey be ? ToString(key).
    2. For each element e of the GlobalSymbolRegistry List,
       a. If SameValue(e.[[Key]], stringKey) is true, return e.[[Symbol]].
    3. Assert: GlobalSymbolRegistry does not currently contain an entry for
       stringKey.
    4. Let newSymbol be a new unique Symbol value whose [[Description]] value
       is stringKey.
    5. Append the Record { [[Key]]: stringKey, [[Symbol]]: newSymbol } to the
       GlobalSymbolRegistry List.
    6. Return newSymbol. 
features: [Symbol]
---*/

var canonical = Symbol.for('s');

assert.sameValue(typeof canonical, 'symbol');
assert.notSameValue(canonical, Symbol('s'));
assert.notSameValue(canonical, Symbol.for('y'));
