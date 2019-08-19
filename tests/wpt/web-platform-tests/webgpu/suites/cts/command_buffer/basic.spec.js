/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
Basic tests.
`;
import { TestGroup } from '../../../framework/index.js';
import { GPUTest } from '../gpu_test.js';
export const g = new TestGroup(GPUTest);
g.test('empty', async t => {
  const encoder = t.device.createCommandEncoder({});
  const cmd = encoder.finish();
  t.device.getQueue().submit([cmd]); // TODO: test that submit() succeeded.
});
//# sourceMappingURL=basic.spec.js.map