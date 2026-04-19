// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.parse
description: >
  JSON.parse reviver is called with the correct arguments when the object is
  modified
includes: [compareArray.js]
features: [json-parse-with-source]
---*/

// Test Array append
{
  let log = [];
  const o = JSON.parse('[1,[]]', function reviver(k, v, { source }) {
    log.push(`key: |${k}| value: ${JSON.stringify(v)} source: |${source}|`);
    if (v === 1) {
      this[1].push('barf');
    }
    return this[k];
  });
  assert.compareArray(log, [
    'key: |0| value: 1 source: |1|',
    'key: |0| value: "barf" source: |undefined|',
    'key: |1| value: ["barf"] source: |undefined|',
    'key: || value: [1,["barf"]] source: |undefined|',
  ]);
}

// Test Object add property
{
  let log = [];
  const o = JSON.parse('{"p":1,"q":{}}', function (k, v, { source }) {
    log.push(`key: |${k}| value: ${JSON.stringify(v)} source: |${source}|`);
    if (v === 1) {
      this.q.added = 'barf';
    }
    return this[k];
  });
  assert.compareArray(log, [
    'key: |p| value: 1 source: |1|',
    'key: |added| value: "barf" source: |undefined|',
    'key: |q| value: {"added":"barf"} source: |undefined|',
    'key: || value: {"p":1,"q":{"added":"barf"}} source: |undefined|',
  ]);
}
