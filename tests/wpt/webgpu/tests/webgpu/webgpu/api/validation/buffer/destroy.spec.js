/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for GPUBuffer.destroy.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kBufferUsages } from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('all_usages').
desc('Test destroying buffers of every usage type.').
paramsSubcasesOnly((u) =>
u //
.combine('usage', kBufferUsages)
).
fn((t) => {
  const { usage } = t.params;
  const buf = t.device.createBuffer({
    size: 4,
    usage
  });

  buf.destroy();
});

g.test('error_buffer').
desc('Test that error buffers may be destroyed without generating validation errors.').
fn((t) => {
  const buf = t.getErrorBuffer();
  buf.destroy();
});

g.test('twice').
desc(
  `Test that destroying a buffer more than once is allowed.
      - Tests buffers which are mapped at creation or not
      - Tests buffers with various usages`
).
paramsSubcasesOnly((u) =>
u //
.combine('mappedAtCreation', [false, true]).
combineWithParams([
{ size: 4, usage: GPUConst.BufferUsage.COPY_SRC },
{ size: 4, usage: GPUConst.BufferUsage.MAP_WRITE | GPUConst.BufferUsage.COPY_SRC },
{ size: 4, usage: GPUConst.BufferUsage.COPY_DST | GPUConst.BufferUsage.MAP_READ }]
)
).
fn((t) => {
  const buf = t.device.createBuffer(t.params);

  buf.destroy();
  buf.destroy();
});

g.test('while_mapped').
desc(
  `Test destroying buffers while mapped or after being unmapped.
      - Tests {mappable, unmappable mapAtCreation, mappable mapAtCreation}
      - Tests while {mapped, mapped at creation, unmapped}`
).
paramsSubcasesOnly((u) =>
u //
.combine('mappedAtCreation', [false, true]).
combine('unmapBeforeDestroy', [false, true]).
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
unless((p) => p.mappedAtCreation === false && p.mapMode === undefined)
).
fn(async (t) => {
  const { usage, mapMode, mappedAtCreation, unmapBeforeDestroy } = t.params;
  const buf = t.device.createBuffer({
    size: 4,
    usage,
    mappedAtCreation
  });

  if (mapMode !== undefined) {
    if (mappedAtCreation) {
      buf.unmap();
    }
    await buf.mapAsync(mapMode);
  }
  if (unmapBeforeDestroy) {
    buf.unmap();
  }

  buf.destroy();
});