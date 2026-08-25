// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.rawjson
description: Basic functionality of JSON.rawJSON().
info: |
  JSON.rawJSON ( text )

  1. Let jsonString be ? ToString(text).
  ...
  4. Let internalSlotsList be « [[IsRawJSON]] ».
  5. Let obj be OrdinaryObjectCreate(null, internalSlotsList).
  6. Perform ! CreateDataPropertyOrThrow(obj, "rawJSON", jsonString).
  7. Perform ! SetIntegrityLevel(obj, frozen).
  8. Return obj.

features: [json-parse-with-source]
---*/

assert.sameValue(JSON.stringify(JSON.rawJSON(1)), '1');
assert.sameValue(JSON.stringify(JSON.rawJSON(1.1)), '1.1');
assert.sameValue(JSON.stringify(JSON.rawJSON(-1)), '-1');
assert.sameValue(JSON.stringify(JSON.rawJSON(-1.1)), '-1.1');
assert.sameValue(JSON.stringify(JSON.rawJSON(1.1e1)), '11');
assert.sameValue(JSON.stringify(JSON.rawJSON(1.1e-1)), '0.11');

assert.sameValue(JSON.stringify(JSON.rawJSON(null)), 'null');
assert.sameValue(JSON.stringify(JSON.rawJSON(true)), 'true');
assert.sameValue(JSON.stringify(JSON.rawJSON(false)), 'false');
assert.sameValue(JSON.stringify(JSON.rawJSON('"foo"')), '"foo"');

assert.sameValue(JSON.stringify({ 42: JSON.rawJSON(37) }), '{"42":37}');
assert.sameValue(
  JSON.stringify({ x: JSON.rawJSON(1), y: JSON.rawJSON(2) }),
  '{"x":1,"y":2}'
);
assert.sameValue(
  JSON.stringify({ x: { x: JSON.rawJSON(1), y: JSON.rawJSON(2) } }),
  '{"x":{"x":1,"y":2}}'
);

assert.sameValue(JSON.stringify([JSON.rawJSON(1), JSON.rawJSON(1.1)]), '[1,1.1]');
assert.sameValue(
  JSON.stringify([
    JSON.rawJSON('"1"'),
    JSON.rawJSON(true),
    JSON.rawJSON(null),
    JSON.rawJSON(false),
  ]),
  '["1",true,null,false]'
);
assert.sameValue(
  JSON.stringify([{ x: JSON.rawJSON(1), y: JSON.rawJSON(1) }]),
  '[{"x":1,"y":1}]'
);
