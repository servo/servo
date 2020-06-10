/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = '';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

function getBufferDesc(usage) {
  return {
    size: Number.MAX_SAFE_INTEGER,
    usage
  };
}

export const g = makeTestGroup(GPUTest);
g.test('mapWriteAsync').fn(async t => {
  const buffer = t.expectGPUError('out-of-memory', () => {
    return t.device.createBuffer(getBufferDesc(GPUBufferUsage.MAP_WRITE));
  });
  t.shouldReject('OperationError', buffer.mapWriteAsync());
});
g.test('mapReadAsync').fn(async t => {
  const buffer = t.expectGPUError('out-of-memory', () => {
    return t.device.createBuffer(getBufferDesc(GPUBufferUsage.MAP_READ));
  });
  t.shouldReject('OperationError', buffer.mapReadAsync());
});
g.test('createBufferMapped').fn(async t => {
  t.shouldThrow('RangeError', () => {
    t.device.createBufferMapped(getBufferDesc(GPUBufferUsage.COPY_SRC));
  });
});
//# sourceMappingURL=map_oom.spec.js.map