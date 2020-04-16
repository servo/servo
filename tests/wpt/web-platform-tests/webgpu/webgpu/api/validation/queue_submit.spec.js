/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
queue submit validation tests.
`;
import { TestGroup } from '../../../common/framework/test_group.js';
import { ValidationTest } from './validation_test.js';
export const g = new TestGroup(ValidationTest);
g.test('submitting with a mapped buffer is disallowed', async t => {
  const buffer = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.MAP_WRITE | GPUBufferUsage.COPY_SRC
  });
  const targetBuffer = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_DST
  });

  const getCommandBuffer = () => {
    const commandEncoder = t.device.createCommandEncoder();
    commandEncoder.copyBufferToBuffer(buffer, 0, targetBuffer, 0, 4);
    return commandEncoder.finish();
  }; // Submitting when the buffer has never been mapped should succeed


  t.queue.submit([getCommandBuffer()]); // Map the buffer, submitting when the buffer is mapped should fail

  await buffer.mapWriteAsync();
  t.queue.submit([]);
  t.expectValidationError(() => {
    t.queue.submit([getCommandBuffer()]);
  }); // Unmap the buffer, queue submit should succeed

  buffer.unmap();
  t.queue.submit([getCommandBuffer()]);
});
//# sourceMappingURL=queue_submit.spec.js.map