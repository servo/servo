/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for the behavior of GPUQueue.onSubmittedWorkDone().

Note that any promise timeouts will be detected by the framework.
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { range } from '../../../common/util/util.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('without_work')
  .desc(`Await onSubmittedWorkDone once without having submitted any work.`)
  .fn(async t => {
    await t.queue.onSubmittedWorkDone();
  });

g.test('with_work')
  .desc(`Await onSubmittedWorkDone once after submitting some work (writeBuffer).`)
  .fn(async t => {
    const buffer = t.device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_DST });
    t.queue.writeBuffer(buffer, 0, new Uint8Array(4));
    await t.queue.onSubmittedWorkDone();
  });

g.test('many,serial')
  .desc(`Await 1000 onSubmittedWorkDone calls in serial.`)
  .fn(async t => {
    for (let i = 0; i < 1000; ++i) {
      await t.queue.onSubmittedWorkDone();
    }
  });

g.test('many,parallel')
  .desc(`Await 1000 onSubmittedWorkDone calls in parallel with Promise.all().`)
  .fn(async t => {
    const promises = range(1000, () => t.queue.onSubmittedWorkDone());
    await Promise.all(promises);
  });

g.test('many,parallel_order')
  .desc(`Issue 200 onSubmittedWorkDone calls and make sure they resolve in the right order.`)
  .fn(async t => {
    const promises = [];
    let lastResolved = -1;
    for (const i of range(200, i => i)) {
      promises.push(
        t.queue.onSubmittedWorkDone().then(() => {
          t.expect(i === lastResolved + 1);
          lastResolved++;
        })
      );
    }
    await Promise.all(promises);
  });
