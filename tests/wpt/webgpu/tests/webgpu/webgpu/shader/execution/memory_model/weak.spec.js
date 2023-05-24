/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for properties of the WebGPU memory model involving two memory locations.
Specifically, the acquire/release ordering provided by WebGPU's barriers can be used to disallow
weak behaviors in several classic memory model litmus tests.`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import {
  MemoryModelTester,
  buildTestShader,
  MemoryType,
  TestType,
  buildResultShader,
  ResultType,
} from './memory_model_setup.js';

export const g = makeTestGroup(GPUTest);

// A reasonable parameter set, determined heuristically.
const memoryModelTestParams = {
  workgroupSize: 256,
  testingWorkgroups: 739,
  maxWorkgroups: 885,
  shufflePct: 0,
  barrierPct: 0,
  memStressPct: 0,
  memStressIterations: 1024,
  memStressStoreFirstPct: 50,
  memStressStoreSecondPct: 50,
  preStressPct: 100,
  preStressIterations: 33,
  preStressStoreFirstPct: 0,
  preStressStoreSecondPct: 100,
  scratchMemorySize: 1408,
  stressLineSize: 4,
  stressTargetLines: 11,
  stressStrategyBalancePct: 0,
  permuteFirst: 109,
  permuteSecond: 419,
  memStride: 2,
  aliasedMemory: false,
  numBehaviors: 4,
};

const workgroupMemoryMessagePassingTestCode = `
  atomicStore(&wg_test_locations[x_0], 1u);
  workgroupBarrier();
  atomicStore(&wg_test_locations[y_0], 1u);
  let r0 = atomicLoad(&wg_test_locations[y_1]);
  workgroupBarrier();
  let r1 = atomicLoad(&wg_test_locations[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

const storageMemoryMessagePassingTestCode = `
  atomicStore(&test_locations.value[x_0], 1u);
  storageBarrier();
  atomicStore(&test_locations.value[y_0], 1u);
  let r0 = atomicLoad(&test_locations.value[y_1]);
  storageBarrier();
  let r1 = atomicLoad(&test_locations.value[x_1]);
  atomicStore(&results.value[shuffled_workgroup * u32(workgroupXSize) + id_1].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * u32(workgroupXSize) + id_1].r1, r1);
`;

g.test('message_passing')
  .desc(
    `Checks whether two reads on one thread can observe two writes in another thread in a way
    that is inconsistent with sequential consistency. In the message passing litmus test, one
    thread writes the value 1 to some location x and then 1 to some location y. The second thread
    reads y and then x. If the second thread reads y == 1 and x == 0, then sequential consistency
    has not been respected. The acquire/release semantics of WebGPU's barrier functions should disallow
    this behavior within a workgroup.
    `
  )
  .paramsSimple([
    { memType: MemoryType.AtomicWorkgroupClass, _testCode: workgroupMemoryMessagePassingTestCode },
    { memType: MemoryType.AtomicStorageClass, _testCode: storageMemoryMessagePassingTestCode },
  ])
  .fn(async t => {
    const testShader = buildTestShader(
      t.params._testCode,
      t.params.memType,
      TestType.IntraWorkgroup
    );

    const messagePassingResultShader = buildResultShader(
      `
      if ((r0 == 0u && r1 == 0u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((r0 == 1u && r1 == 1u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((r0 == 0u && r1 == 1u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((r0 == 1u && r1 == 0u)) {
        atomicAdd(&test_results.weak, 1u);
      }
      `,
      TestType.IntraWorkgroup,
      ResultType.FourBehavior
    );

    const memModelTester = new MemoryModelTester(
      t,
      memoryModelTestParams,
      testShader,
      messagePassingResultShader
    );

    await memModelTester.run(40, 3);
  });

const workgroupMemoryStoreTestCode = `
  atomicStore(&wg_test_locations[x_0], 2u);
  workgroupBarrier();
  atomicStore(&wg_test_locations[y_0], 1u);
  let r0 = atomicLoad(&wg_test_locations[y_1]);
  workgroupBarrier();
  atomicStore(&wg_test_locations[x_1], 1u);
  workgroupBarrier();
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1], atomicLoad(&wg_test_locations[x_1]));
`;

const storageMemoryStoreTestCode = `
  atomicStore(&test_locations.value[x_0], 2u);
  storageBarrier();
  atomicStore(&test_locations.value[y_0], 1u);
  let r0 = atomicLoad(&test_locations.value[y_1]);
  storageBarrier();
  atomicStore(&test_locations.value[x_1], 1u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
`;

g.test('store')
  .desc(
    `In the store litmus test, one thread writes 2 to some memory location x and then 1 to some memory location
     y. A second thread reads the value of y and then writes 1 to x. If the read on the second thread returns 1,
     but the value of x in memory after the test ends is 2, then there has been a re-ordering which is not allowed
     when using WebGPU's barriers.
    `
  )
  .paramsSimple([
    { memType: MemoryType.AtomicWorkgroupClass, _testCode: workgroupMemoryStoreTestCode },
    { memType: MemoryType.AtomicStorageClass, _testCode: storageMemoryStoreTestCode },
  ])
  .fn(async t => {
    const testShader = buildTestShader(
      t.params._testCode,
      t.params.memType,
      TestType.IntraWorkgroup
    );

    const messagePassingResultShader = buildResultShader(
      `
      if ((r0 == 1u && mem_x_0 == 1u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((r0 == 0u && mem_x_0 == 2u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((r0 == 0u && mem_x_0 == 1u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((r0 == 1u && mem_x_0 == 2u)) {
        atomicAdd(&test_results.weak, 1u);
      }
      `,
      TestType.IntraWorkgroup,
      ResultType.FourBehavior
    );

    const memModelTester = new MemoryModelTester(
      t,
      memoryModelTestParams,
      testShader,
      messagePassingResultShader
    );

    await memModelTester.run(40, 3);
  });

const workgroupMemoryLoadBufferTestCode = `
  let r0 = atomicLoad(&wg_test_locations[y_0]);
  workgroupBarrier();
  atomicStore(&wg_test_locations[x_0], 1u);
  let r1 = atomicLoad(&wg_test_locations[x_1]);
  workgroupBarrier();
  atomicStore(&wg_test_locations[y_1], 1u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

const storageMemoryLoadBufferTestCode = `
  let r0 = atomicLoad(&test_locations.value[y_0]);
  storageBarrier();
  atomicStore(&test_locations.value[x_0], 1u);
  let r1 = atomicLoad(&test_locations.value[x_1]);
  storageBarrier();
  atomicStore(&test_locations.value[y_1], 1u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

g.test('load_buffer')
  .desc(
    `In the load buffer litmus test, one thread reads from memory location y and then writes 1 to memory location x.
     A second thread reads from x and then writes 1 to y. If both threads read the value 0, then the loads have been
     buffered or re-ordered, which is not allowed when used in conjunction with WebGPU's barriers.
    `
  )
  .paramsSimple([
    { memType: MemoryType.AtomicWorkgroupClass, _testCode: workgroupMemoryLoadBufferTestCode },
    { memType: MemoryType.AtomicStorageClass, _testCode: storageMemoryLoadBufferTestCode },
  ])
  .fn(async t => {
    const testShader = buildTestShader(
      t.params._testCode,
      t.params.memType,
      TestType.IntraWorkgroup
    );

    const messagePassingResultShader = buildResultShader(
      `
      if ((r0 == 1u && r1 == 0u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((r0 == 0u && r1 == 1u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((r0 == 0u && r1 == 0u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((r0 == 1u && r1 == 1u)) {
        atomicAdd(&test_results.weak, 1u);
      }
      `,
      TestType.IntraWorkgroup,
      ResultType.FourBehavior
    );

    const memModelTester = new MemoryModelTester(
      t,
      memoryModelTestParams,
      testShader,
      messagePassingResultShader
    );

    await memModelTester.run(40, 3);
  });

const workgroupMemoryReadTestCode = `
  atomicStore(&wg_test_locations[x_0], 1u);
  workgroupBarrier();
  atomicExchange(&wg_test_locations[y_0], 1u);
  atomicExchange(&wg_test_locations[y_1], 2u);
  workgroupBarrier();
  let r0 = atomicLoad(&wg_test_locations[x_1]);
  workgroupBarrier();
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + y_1], atomicLoad(&wg_test_locations[y_1]));
`;

const storageMemoryReadTestCode = `
  atomicStore(&test_locations.value[x_0], 1u);
  storageBarrier();
  atomicExchange(&test_locations.value[y_0], 1u);
  atomicExchange(&test_locations.value[y_1], 2u);
  storageBarrier();
  let r0 = atomicLoad(&test_locations.value[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
`;

g.test('read')
  .desc(
    `In the read litmus test, one thread writes 1 to memory location x and then 1 to memory location y. A second thread
     first writes 2 to y and then reads from x. If the value read by the second thread is 0 but the value in memory of y
     after the test completes is 2, then there has been some re-ordering of instructions disallowed when using WebGPU's
     barrier. Additionally, both writes to y are RMWs, so that the barrier forces the correct acquire/release memory ordering
     synchronization.
    `
  )
  .paramsSimple([
    { memType: MemoryType.AtomicWorkgroupClass, _testCode: workgroupMemoryReadTestCode },
    { memType: MemoryType.AtomicStorageClass, _testCode: storageMemoryReadTestCode },
  ])
  .fn(async t => {
    const testShader = buildTestShader(
      t.params._testCode,
      t.params.memType,
      TestType.IntraWorkgroup
    );

    const messagePassingResultShader = buildResultShader(
      `
      if ((r0 == 1u && mem_y_0 == 2u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((r0 == 0u && mem_y_0 == 1u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((r0 == 1u && mem_y_0 == 1u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((r0 == 0u && mem_y_0 == 2u)) {
        atomicAdd(&test_results.weak, 1u);
      }
      `,
      TestType.IntraWorkgroup,
      ResultType.FourBehavior
    );

    const memModelTester = new MemoryModelTester(
      t,
      memoryModelTestParams,
      testShader,
      messagePassingResultShader
    );

    await memModelTester.run(40, 3);
  });

const workgroupMemoryStoreBufferTestCode = `
  atomicStore(&wg_test_locations[x_0], 1u);
  workgroupBarrier();
  let r0 = atomicAdd(&wg_test_locations[y_0], 0u);
  atomicExchange(&wg_test_locations[y_1], 1u);
  workgroupBarrier();
  let r1 = atomicLoad(&wg_test_locations[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

const storageMemoryStoreBufferTestCode = `
  atomicStore(&test_locations.value[x_0], 1u);
  storageBarrier();
  let r0 = atomicAdd(&test_locations.value[y_0], 0u);
  atomicExchange(&test_locations.value[y_1], 1u);
  storageBarrier();
  let r1 = atomicLoad(&test_locations.value[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

g.test('store_buffer')
  .desc(
    `In the store buffer litmus test, one thread writes 1 to memory location x and then reads from memory location
     y. A second thread writes 1 to y and then reads from x. If both reads return 0, then stores have been buffered
     or some other re-ordering has occurred that is disallowed by WebGPU's barriers. Additionally, both the read
     and store to y are RMWs to achieve the necessary synchronization across threads.
    `
  )
  .paramsSimple([
    { memType: MemoryType.AtomicWorkgroupClass, _testCode: workgroupMemoryStoreBufferTestCode },
    { memType: MemoryType.AtomicStorageClass, _testCode: storageMemoryStoreBufferTestCode },
  ])
  .fn(async t => {
    const testShader = buildTestShader(
      t.params._testCode,
      t.params.memType,
      TestType.IntraWorkgroup
    );

    const messagePassingResultShader = buildResultShader(
      `
      if ((r0 == 1u && r1 == 0u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((r0 == 0u && r1 == 1u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((r0 == 1u && r1 == 1u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((r0 == 0u && r1 == 0u)) {
        atomicAdd(&test_results.weak, 1u);
      }
      `,
      TestType.IntraWorkgroup,
      ResultType.FourBehavior
    );

    const memModelTester = new MemoryModelTester(
      t,
      memoryModelTestParams,
      testShader,
      messagePassingResultShader
    );

    await memModelTester.run(40, 3);
  });

const workgroupMemory2P2WTestCode = `
  atomicStore(&wg_test_locations[x_0], 2u);
  workgroupBarrier();
  atomicExchange(&wg_test_locations[y_0], 1u);
  atomicExchange(&wg_test_locations[y_1], 2u);
  workgroupBarrier();
  atomicStore(&wg_test_locations[x_1], 1u);
  workgroupBarrier();
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1], atomicLoad(&wg_test_locations[x_1]));
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + y_1], atomicLoad(&wg_test_locations[y_1]));
`;

const storageMemory2P2WTestCode = `
  atomicStore(&test_locations.value[x_0], 2u);
  storageBarrier();
  atomicExchange(&test_locations.value[y_0], 1u);
  atomicExchange(&test_locations.value[y_1], 2u);
  storageBarrier();
  atomicStore(&test_locations.value[x_1], 1u);
`;

g.test('2_plus_2_write')
  .desc(
    `In the 2+2 write litmus test, one thread stores 2 to memory location x and then 1 to memory location y.
     A second thread stores 2 to y and then 1 to x. If at the end of the test both memory locations are set to 2,
     then some disallowed re-ordering has occurred. Both writes to y are RMWs to achieve the required synchronization.
    `
  )
  .paramsSimple([
    { memType: MemoryType.AtomicWorkgroupClass, _testCode: workgroupMemory2P2WTestCode },
    { memType: MemoryType.AtomicStorageClass, _testCode: storageMemory2P2WTestCode },
  ])
  .fn(async t => {
    const testShader = buildTestShader(
      t.params._testCode,
      t.params.memType,
      TestType.IntraWorkgroup
    );

    const messagePassingResultShader = buildResultShader(
      `
      if ((mem_x_0 == 1u && mem_y_0 == 2u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((mem_x_0 == 2u && mem_y_0 == 1u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((mem_x_0 == 1u && mem_y_0 == 1u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((mem_x_0 == 2u && mem_y_0 == 2u)) {
        atomicAdd(&test_results.weak, 1u);
      }
      `,
      TestType.IntraWorkgroup,
      ResultType.FourBehavior
    );

    const memModelTester = new MemoryModelTester(
      t,
      memoryModelTestParams,
      testShader,
      messagePassingResultShader
    );

    await memModelTester.run(40, 3);
  });
