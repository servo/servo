/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = ``;
import { TestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

function getBufferDesc() {
  return {
    size: Number.MAX_SAFE_INTEGER,
    usage: GPUBufferUsage.MAP_WRITE
  };
}

export const g = new TestGroup(GPUTest);
g.test('mapWriteAsync', async t => {
  const buffer = t.device.createBuffer(getBufferDesc());
  t.shouldReject('RangeError', buffer.mapWriteAsync());
});
g.test('mapReadAsync', async t => {
  const buffer = t.device.createBuffer(getBufferDesc());
  t.shouldReject('RangeError', buffer.mapReadAsync());
});
g.test('createBufferMapped', async t => {
  t.shouldThrow('RangeError', () => {
    t.device.createBufferMapped(getBufferDesc());
  });
});
//# sourceMappingURL=map_oom.spec.js.map