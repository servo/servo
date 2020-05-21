/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = '';
import { pbool, poptions, params } from '../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { MappingTest } from './mapping_test.js';
export const g = makeTestGroup(MappingTest);
g.test('mapWriteAsync').params(poptions('size', [12, 512 * 1024])).fn(async t => {
  const {
    size
  } = t.params;
  const buffer = t.device.createBuffer({
    size,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE
  });
  const arrayBuffer = await buffer.mapWriteAsync();
  t.checkMapWrite(buffer, arrayBuffer, size);
});
g.test('mapReadAsync').params(poptions('size', [12, 512 * 1024])).fn(async t => {
  const {
    size
  } = t.params;
  const [buffer, init] = t.device.createBufferMapped({
    size,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
  });
  const expected = new Uint32Array(new ArrayBuffer(size));
  const data = new Uint32Array(init);

  for (let i = 0; i < data.length; ++i) {
    data[i] = expected[i] = i + 1;
  }

  buffer.unmap();
  const actual = new Uint8Array((await buffer.mapReadAsync()));
  t.expectBuffer(actual, new Uint8Array(expected.buffer));
});
g.test('createBufferMapped').params(params().combine(poptions('size', [12, 512 * 1024])).combine(pbool('mappable'))).fn(async t => {
  const {
    size,
    mappable
  } = t.params;
  const [buffer, arrayBuffer] = t.device.createBufferMapped({
    size,
    usage: GPUBufferUsage.COPY_SRC | (mappable ? GPUBufferUsage.MAP_WRITE : 0)
  });
  t.checkMapWrite(buffer, arrayBuffer, size);
});
//# sourceMappingURL=map.spec.js.map