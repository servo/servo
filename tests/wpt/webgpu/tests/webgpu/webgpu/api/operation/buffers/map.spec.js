/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = '';
import { pbool, params } from '../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/framework/util/util.js';

import { MappingTest } from './mapping_test.js';

export const g = makeTestGroup(MappingTest);

const kCases = [
  { size: 0, range: [] },
  { size: 0, range: [undefined] },
  { size: 0, range: [undefined, undefined] },
  { size: 0, range: [0] },
  { size: 0, range: [0, undefined] },
  { size: 0, range: [0, 0] },
  { size: 12, range: [] },
  { size: 12, range: [undefined] },
  { size: 12, range: [undefined, undefined] },
  { size: 12, range: [0] },
  { size: 12, range: [0, undefined] },
  { size: 12, range: [0, 12] },
  { size: 12, range: [0, 0] },
  { size: 12, range: [8] },
  { size: 12, range: [8, undefined] },
  { size: 12, range: [8, 4] },
  { size: 28, range: [8, 8] },
  { size: 28, range: [8, 12] },
  { size: 512 * 1024, range: [] },
];

function reifyMapRange(bufferSize, range) {
  var _range$, _range$2;
  const offset = (_range$ = range[0]) !== null && _range$ !== void 0 ? _range$ : 0;
  return [
    offset,
    (_range$2 = range[1]) !== null && _range$2 !== void 0 ? _range$2 : bufferSize - offset,
  ];
}

g.test('mapAsync,write')
  .params(kCases)
  .fn(async t => {
    const { size, range } = t.params;
    const [rangeOffset, rangeSize] = reifyMapRange(size, range);

    const buffer = t.device.createBuffer({
      size,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE,
    });

    await buffer.mapAsync(GPUMapMode.WRITE);
    const arrayBuffer = buffer.getMappedRange(...range);
    t.checkMapWrite(buffer, rangeOffset, arrayBuffer, rangeSize);
  });

g.test('mapAsync,read')
  .params(kCases)
  .fn(async t => {
    const { size, range } = t.params;
    const [, rangeSize] = reifyMapRange(size, range);

    const buffer = t.device.createBuffer({
      mappedAtCreation: true,
      size,
      usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    });

    const init = buffer.getMappedRange(...range);

    assert(init.byteLength === rangeSize);
    const expected = new Uint32Array(new ArrayBuffer(rangeSize));
    const data = new Uint32Array(init);
    for (let i = 0; i < data.length; ++i) {
      data[i] = expected[i] = i + 1;
    }
    buffer.unmap();

    await buffer.mapAsync(GPUMapMode.READ);
    const actual = new Uint8Array(buffer.getMappedRange(...range));
    t.expectBuffer(actual, new Uint8Array(expected.buffer));
  });

g.test('mappedAtCreation')
  .params(
    params()
      .combine(kCases) //
      .combine(pbool('mappable'))
  )
  .fn(async t => {
    var _range$3;
    const { size, range, mappable } = t.params;
    const [, rangeSize] = reifyMapRange(size, range);

    const buffer = t.device.createBuffer({
      mappedAtCreation: true,
      size,
      usage: GPUBufferUsage.COPY_SRC | (mappable ? GPUBufferUsage.MAP_WRITE : 0),
    });

    const arrayBuffer = buffer.getMappedRange(...range);
    t.checkMapWrite(
      buffer,
      (_range$3 = range[0]) !== null && _range$3 !== void 0 ? _range$3 : 0,
      arrayBuffer,
      rangeSize
    );
  });
