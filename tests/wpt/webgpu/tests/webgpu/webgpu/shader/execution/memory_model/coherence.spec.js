/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that all threads see a sequentially consistent view of the order of memory
accesses to a single memory location. Uses a parallel testing strategy along with stressing
threads to increase coverage of possible bugs.`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import {

  MemoryModelTester,
  buildTestShader,
  MemoryType,
  TestType,
  buildResultShader,
  ResultType } from
'./memory_model_setup.js';

export const g = makeTestGroup(GPUTest);

// A reasonable parameter set, determined heuristically.
const memoryModelTestParams = {
  workgroupSize: 256,
  testingWorkgroups: 39,
  maxWorkgroups: 952,
  shufflePct: 0,
  barrierPct: 0,
  memStressPct: 0,
  memStressIterations: 1024,
  memStressStoreFirstPct: 50,
  memStressStoreSecondPct: 50,
  preStressPct: 0,
  preStressIterations: 1024,
  preStressStoreFirstPct: 50,
  preStressStoreSecondPct: 50,
  scratchMemorySize: 2048,
  stressLineSize: 64,
  stressTargetLines: 2,
  stressStrategyBalancePct: 50,
  permuteFirst: 109,
  permuteSecond: 1,
  memStride: 1,
  aliasedMemory: true,
  numBehaviors: 4
};

const storageMemoryCorrTestCode = `
  atomicStore(&test_locations.value[x_0], 1u);
  let r0 = atomicLoad(&test_locations.value[x_1]);
  let r1 = atomicLoad(&test_locations.value[y_1]);
  atomicStore(&results.value[id_1].r0, r0);
  atomicStore(&results.value[id_1].r1, r1);
`;

const workgroupStorageMemoryCorrTestCode = `
  atomicStore(&test_locations.value[x_0], 1u);
  let r0 = atomicLoad(&test_locations.value[x_1]);
  let r1 = atomicLoad(&test_locations.value[y_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

const storageMemoryCorrRMWTestCode = `
  atomicExchange(&test_locations.value[x_0], 1u);
  let r0 = atomicLoad(&test_locations.value[x_1]);
  let r1 = atomicAdd(&test_locations.value[y_1], 0u);
  atomicStore(&results.value[id_1].r0, r0);
  atomicStore(&results.value[id_1].r1, r1);
`;

const workgroupStorageMemoryCorrRMWTestCode = `
  atomicExchange(&test_locations.value[x_0], 1u);
  let r0 = atomicLoad(&test_locations.value[x_1]);
  let r1 = atomicAdd(&test_locations.value[y_1], 0u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

const workgroupMemoryCorrTestCode = `
  atomicStore(&wg_test_locations[x_0], 1u);
  let r0 = atomicLoad(&wg_test_locations[x_1]);
  let r1 = atomicLoad(&wg_test_locations[y_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

const workgroupMemoryCorrRMWTestCode = `
  atomicExchange(&wg_test_locations[x_0], 1u);
  let r0 = atomicLoad(&wg_test_locations[x_1]);
  let r1 = atomicAdd(&wg_test_locations[y_1], 0u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r1, r1);
`;

g.test('corr').
desc(
  `Ensures two reads on one thread cannot observe an inconsistent view of a write on a second thread.
     The first thread writes the value 1 some location x, and the second thread reads x twice in a row.
     If the first read returns 1 but the second read returns 0, then there has been a coherence violation.
    `
).
paramsSimple([
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCorrTestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCorrRMWTestCode,
  extraFlags: 'rmw_variant'
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupStorageMemoryCorrTestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupStorageMemoryCorrRMWTestCode,
  extraFlags: 'rmw_variant'
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCorrTestCode
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCorrRMWTestCode,
  extraFlags: 'rmw_variant'
}]
).
fn(async (t) => {
  const resultCode = `
      if ((r0 == 0u && r1 == 0u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((r0 == 1u && r1 == 1u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((r0 == 0u && r1 == 1u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((r0 == 1u && r1 == 0u)) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  const testShader = buildTestShader(t.params._testCode, t.params.memType, t.params.testType);
  const resultShader = buildResultShader(resultCode, t.params.testType, ResultType.FourBehavior);
  const memModelTester = new MemoryModelTester(
    t,
    memoryModelTestParams,
    testShader,
    resultShader
  );
  await memModelTester.run(60, 3);
});

const storageMemoryCowwTestCode = `
  atomicStore(&test_locations.value[x_0], 1u);
  atomicStore(&test_locations.value[y_0], 2u);
`;

const storageMemoryCowwRMWTestCode = `
  atomicExchange(&test_locations.value[x_0], 1u);
  atomicStore(&test_locations.value[y_0], 2u);
`;

const workgroupMemoryCowwTestCode = `
  atomicStore(&wg_test_locations[x_0], 1u);
  atomicStore(&wg_test_locations[y_0], 2u);
  workgroupBarrier();
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_0], atomicLoad(&wg_test_locations[x_0]));
`;

const workgroupMemoryCowwRMWTestCode = `
  atomicExchange(&wg_test_locations[x_0], 1u);
  atomicStore(&wg_test_locations[y_0], 2u);
  workgroupBarrier();
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_0], atomicLoad(&wg_test_locations[x_0]));
`;

g.test('coww').
desc(
  `Ensures two writes on one thread do not lead to incoherent results. The thread first writes 1 to
     some location x and then writes 2 to the same location. If the value in memory after the test finishes
     is 1, then there has been a coherence violation.
    `
).
paramsSimple([
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCowwTestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCowwRMWTestCode,
  extraFlags: 'rmw_variant'
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: storageMemoryCowwTestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: storageMemoryCowwRMWTestCode,
  extraFlags: 'rmw_variant'
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCowwTestCode
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCowwRMWTestCode,
  extraFlags: 'rmw_variant'
}]
).
fn(async (t) => {
  const resultCode = `
      if (mem_x_0 == 2u) {
        atomicAdd(&test_results.seq, 1u);
      } else if (mem_x_0 == 1u) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  const testShader = buildTestShader(t.params._testCode, t.params.memType, t.params.testType);
  const resultShader = buildResultShader(resultCode, t.params.testType, ResultType.TwoBehavior);
  const params = {
    ...memoryModelTestParams,
    numBehaviors: 2
  };
  const memModelTester = new MemoryModelTester(t, params, testShader, resultShader);
  await memModelTester.run(60, 1);
});

const storageMemoryCowrTestCode = `
  atomicStore(&test_locations.value[x_0], 1u);
  let r0 = atomicLoad(&test_locations.value[y_0]);
  atomicStore(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[id_0].r0, r0);
`;

const workgroupStorageMemoryCowrTestCode = `
  atomicStore(&test_locations.value[x_0], 1u);
  let r0 = atomicLoad(&test_locations.value[y_0]);
  atomicStore(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const storageMemoryCowrRMWTestCode = `
  atomicExchange(&test_locations.value[x_0], 1u);
  let r0 = atomicAdd(&test_locations.value[y_0], 0u);
  atomicExchange(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[id_0].r0, r0);
`;

const workgroupStorageMemoryCowrRMWTestCode = `
  atomicExchange(&test_locations.value[x_0], 1u);
  let r0 = atomicAdd(&test_locations.value[y_0], 0u);
  atomicExchange(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const workgroupMemoryCowrTestCode = `
  atomicStore(&wg_test_locations[x_0], 1u);
  let r0 = atomicLoad(&wg_test_locations[y_0]);
  atomicStore(&wg_test_locations[x_1], 2u);
  workgroupBarrier();
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1], atomicLoad(&wg_test_locations[x_1]));
`;

const workgroupMemoryCowrRMWTestCode = `
  atomicExchange(&wg_test_locations[x_0], 1u);
  let r0 = atomicAdd(&wg_test_locations[y_0], 0u);
  atomicExchange(&wg_test_locations[x_1], 2u);
  workgroupBarrier();
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1], atomicLoad(&wg_test_locations[x_1]));
`;

g.test('cowr').
desc(
  `The first thread first writes 1 to some location x and then reads x. The second thread writes 2 to x.
     If the first thread reads the value 2 and the value in memory at the end of the test is 1, then the read
     and write on the first thread have been reordered, a coherence violation.
    `
).
paramsSimple([
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCowrTestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCowrRMWTestCode,
  extraFlags: 'rmw_variant'
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupStorageMemoryCowrTestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupStorageMemoryCowrRMWTestCode,
  extraFlags: 'rmw_variant'
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCowrTestCode
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCowrRMWTestCode,
  extraFlags: 'rmw_variant'
}]
).
fn(async (t) => {
  const resultCode = `
      if ((r0 == 1u && mem_x_0 == 2u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((r0 == 1u && mem_x_0 == 1u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((r0 == 2u && mem_x_0 == 2u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((r0 == 2u && mem_x_0 == 1u)) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  const testShader = buildTestShader(t.params._testCode, t.params.memType, t.params.testType);
  const resultShader = buildResultShader(resultCode, t.params.testType, ResultType.FourBehavior);
  const memModelTester = new MemoryModelTester(
    t,
    memoryModelTestParams,
    testShader,
    resultShader
  );
  await memModelTester.run(60, 3);
});

const storageMemoryCorw1TestCode = `
  let r0 = atomicLoad(&test_locations.value[x_0]);
  atomicStore(&test_locations.value[x_0], 1u);
  workgroupBarrier();
  atomicStore(&results.value[id_0].r0, r0);
`;

const workgroupStorageMemoryCorw1TestCode = `
  let r0 = atomicLoad(&test_locations.value[x_0]);
  atomicStore(&test_locations.value[y_0], 1u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const workgroupMemoryCorw1TestCode = `
  let r0 = atomicLoad(&wg_test_locations[x_0]);
  atomicStore(&wg_test_locations[y_0], 1u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

g.test('corw1').
desc(
  `One thread first reads from a memory location x and then writes 1 to x. If the read observes the subsequent
     write, there has been a coherence violation.
    `
).
paramsSimple([
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCorw1TestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupStorageMemoryCorw1TestCode
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCorw1TestCode
}]
).
fn(async (t) => {
  const resultCode = `
      if (r0 == 0u) {
        atomicAdd(&test_results.seq, 1u);
      } else if (r0 == 1u) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  const testShader = buildTestShader(t.params._testCode, t.params.memType, t.params.testType);
  const resultShader = buildResultShader(resultCode, t.params.testType, ResultType.TwoBehavior);
  const params = {
    ...memoryModelTestParams,
    numBehaviors: 2
  };
  const memModelTester = new MemoryModelTester(t, params, testShader, resultShader);
  await memModelTester.run(60, 1);
});

const storageMemoryCorw2TestCode = `
  let r0 = atomicLoad(&test_locations.value[x_0]);
  atomicStore(&test_locations.value[y_0], 1u);
  atomicStore(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[id_0].r0, r0);
`;

const workgroupStorageMemoryCorw2TestCode = `
  let r0 = atomicLoad(&test_locations.value[x_0]);
  atomicStore(&test_locations.value[y_0], 1u);
  atomicStore(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const storageMemoryCorw2RMWTestCode = `
  let r0 = atomicLoad(&test_locations.value[x_0]);
  atomicStore(&test_locations.value[y_0], 1u);
  atomicExchange(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[id_0].r0, r0);
`;

const workgroupStorageMemoryCorw2RMWTestCode = `
  let r0 = atomicLoad(&test_locations.value[x_0]);
  atomicStore(&test_locations.value[y_0], 1u);
  atomicExchange(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const workgroupMemoryCorw2TestCode = `
  let r0 = atomicLoad(&wg_test_locations[x_0]);
  atomicStore(&wg_test_locations[y_0], 1u);
  atomicStore(&wg_test_locations[x_1], 2u);
  workgroupBarrier();
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1], atomicLoad(&wg_test_locations[x_1]));
`;

const workgroupMemoryCorw2RMWTestCode = `
  let r0 = atomicLoad(&wg_test_locations[x_0]);
  atomicStore(&wg_test_locations[y_0], 1u);
  atomicExchange(&wg_test_locations[x_1], 2u);
  workgroupBarrier();
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1], atomicLoad(&wg_test_locations[x_1]));
`;

g.test('corw2').
desc(
  `The first thread reads from some memory location x, and then writes 1 to x. The second thread
     writes 2 to x. If the first thread reads the value 2, but the value in memory after the test
     completes is 1, then the instructions on the first thread have been re-ordered, leading to a
     coherence violation.
    `
).
paramsSimple([
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCorw2TestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.InterWorkgroup,
  _testCode: storageMemoryCorw2RMWTestCode,
  extraFlags: 'rmw_variant'
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupStorageMemoryCorw2TestCode
},
{
  memType: MemoryType.AtomicStorageClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupStorageMemoryCorw2RMWTestCode,
  extraFlags: 'rmw_variant'
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCorw2TestCode
},
{
  memType: MemoryType.AtomicWorkgroupClass,
  testType: TestType.IntraWorkgroup,
  _testCode: workgroupMemoryCorw2RMWTestCode,
  extraFlags: 'rmw_variant'
}]
).
fn(async (t) => {
  const resultCode = `
      if ((r0 == 0u && mem_x_0 == 2u)) {
        atomicAdd(&test_results.seq0, 1u);
      } else if ((r0 == 2u && mem_x_0 == 1u)) {
        atomicAdd(&test_results.seq1, 1u);
      } else if ((r0 == 0u && mem_x_0 == 1u)) {
        atomicAdd(&test_results.interleaved, 1u);
      } else if ((r0 == 2u && mem_x_0 == 2u)) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  const testShader = buildTestShader(t.params._testCode, t.params.memType, t.params.testType);
  const resultShader = buildResultShader(resultCode, t.params.testType, ResultType.FourBehavior);
  const memModelTester = new MemoryModelTester(
    t,
    memoryModelTestParams,
    testShader,
    resultShader
  );
  await memModelTester.run(60, 3);
});