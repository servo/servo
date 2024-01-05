/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = 'Test out-of-memory conditions creating large mappable/mappedAtCreation buffers.';
import { kUnitCaseParamsBuilder } from '../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kBufferUsages } from '../../../capability_info.js';
import { GPUTest } from '../../../gpu_test.js';
import { kMaxSafeMultipleOf8 } from '../../../util/math.js';

const oomAndSizeParams = kUnitCaseParamsBuilder.
combine('oom', [false, true]).
expand('size', ({ oom }) => {
  return oom ?
  [
  kMaxSafeMultipleOf8,
  0x20_0000_0000 // 128 GB
  ] :
  [16];
});

export const g = makeTestGroup(GPUTest);

g.test('mappedAtCreation').
desc(
  `Test creating a very large buffer mappedAtCreation buffer should throw a RangeError only
     because such a large allocation cannot be created when we initialize an active buffer mapping.
`
).
params(
  oomAndSizeParams //
  .beginSubcases().
  combine('usage', kBufferUsages)
).
fn((t) => {
  const { oom, usage, size } = t.params;

  const f = () => t.device.createBuffer({ mappedAtCreation: true, size, usage });

  if (oom) {
    // getMappedRange is normally valid on OOM buffers, but this one fails because the
    // (default) range is too large to create the returned ArrayBuffer.
    t.shouldThrow('RangeError', f);
  } else {
    const buffer = f();
    const mapping = buffer.getMappedRange();
    t.expect(mapping.byteLength === size, 'Mapping should be successful');
    buffer.unmap();
    t.expect(mapping.byteLength === 0, 'Mapping should be detached');
  }
});