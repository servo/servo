/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
  Tests that TypedArrays created when mapping a GPUBuffer are detached when the
  buffer is unmapped or destroyed.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { getGPU } from '../../../../common/util/navigator_gpu.js';
import { assert } from '../../../../common/util/util.js';
import { GPUConst } from '../../../constants.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('while_mapped').
desc(
  `
    Test that a mapped buffers are able to properly detach.
    - Tests {mappable, unmappable mapAtCreation, mappable mapAtCreation}
    - Tests while {mapped, mapped at creation, mapped at creation then unmapped and mapped again}
    - When {unmap, destroy, unmap && destroy, device.destroy} is called`
).
paramsSubcasesOnly((u) =>
u.
combine('mappedAtCreation', [false, true]).
combineWithParams([
{ usage: GPUConst.BufferUsage.COPY_SRC },
{ usage: GPUConst.BufferUsage.MAP_WRITE | GPUConst.BufferUsage.COPY_SRC },
{ usage: GPUConst.BufferUsage.COPY_DST | GPUConst.BufferUsage.MAP_READ },
{
  usage: GPUConst.BufferUsage.MAP_WRITE | GPUConst.BufferUsage.COPY_SRC,
  mapMode: GPUConst.MapMode.WRITE
},
{
  usage: GPUConst.BufferUsage.COPY_DST | GPUConst.BufferUsage.MAP_READ,
  mapMode: GPUConst.MapMode.READ
}]
).
combineWithParams([
{ unmap: true, destroy: false },
{ unmap: false, destroy: true },
{ unmap: true, destroy: true },
{ unmap: false, destroy: false, deviceDestroy: true }]
).
unless((p) => p.mappedAtCreation === false && p.mapMode === undefined)
).
fn(async (t) => {
  const { usage, mapMode, mappedAtCreation, unmap, destroy, deviceDestroy } = t.params;

  let device = t.device;
  if (deviceDestroy) {
    const adapter = await getGPU(t.rec).requestAdapter();
    assert(adapter !== null);
    device = await adapter.requestDevice();
  }
  const buffer = device.createBuffer({
    size: 4,
    usage,
    mappedAtCreation
  });

  if (mapMode !== undefined) {
    if (mappedAtCreation) {
      buffer.unmap();
    }
    await buffer.mapAsync(mapMode);
  }

  const arrayBuffer = buffer.getMappedRange();
  const view = new Uint8Array(arrayBuffer);
  t.expect(arrayBuffer.byteLength === 4);
  t.expect(view.length === 4);

  if (unmap) buffer.unmap();
  if (destroy) buffer.destroy();
  if (deviceDestroy) device.destroy();

  t.expect(arrayBuffer.byteLength === 0, 'ArrayBuffer should be detached');
  t.expect(view.byteLength === 0, 'ArrayBufferView should be detached');
});