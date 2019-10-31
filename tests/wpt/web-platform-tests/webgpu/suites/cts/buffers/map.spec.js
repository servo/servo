/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = ``;
import { TestGroup, pbool, pcombine, poptions } from '../../../framework/index.js';
import { MappingTest } from './mapping_test.js';
export const g = new TestGroup(MappingTest);
g.test('mapWriteAsync', async t => {
  const size = t.params.size;
  const buffer = t.device.createBuffer({
    size,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE
  });
  const arrayBuffer = await buffer.mapWriteAsync();
  t.checkMapWrite(buffer, arrayBuffer, size);
}).params(poptions('size', [12, 512 * 1024]));
g.test('mapReadAsync', async t => {
  const size = t.params.size;
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
}).params(poptions('size', [12, 512 * 1024]));
g.test('createBufferMapped', async t => {
  const size = t.params.size;
  const [buffer, arrayBuffer] = t.device.createBufferMapped({
    size,
    usage: GPUBufferUsage.COPY_SRC | (t.params.mappable ? GPUBufferUsage.MAP_WRITE : 0)
  });
  t.checkMapWrite(buffer, arrayBuffer, size);
}).params(pcombine(poptions('size', [12, 512 * 1024]), //
pbool('mappable')));
g.test('createBufferMappedAsync', async t => {
  const size = t.params.size;
  const [buffer, arrayBuffer] = await t.device.createBufferMappedAsync({
    size,
    usage: GPUBufferUsage.COPY_SRC | (t.params.mappable ? GPUBufferUsage.MAP_WRITE : 0)
  });
  t.checkMapWrite(buffer, arrayBuffer, size);
}).params(pcombine(poptions('size', [12, 512 * 1024]), //
pbool('mappable')));
//# sourceMappingURL=map.spec.js.map