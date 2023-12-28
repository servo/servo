/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for non-atomic memory synchronization within a workgroup in the presence of a WebGPU barrier`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import {

  MemoryModelTester,
  kAccessValueTypes,
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
  testingWorkgroups: 512,
  maxWorkgroups: 1024,
  shufflePct: 100,
  barrierPct: 100,
  memStressPct: 100,
  memStressIterations: 1024,
  memStressStoreFirstPct: 50,
  memStressStoreSecondPct: 50,
  preStressPct: 100,
  preStressIterations: 1024,
  preStressStoreFirstPct: 50,
  preStressStoreSecondPct: 50,
  scratchMemorySize: 2048,
  stressLineSize: 64,
  stressTargetLines: 2,
  stressStrategyBalancePct: 50,
  permuteFirst: 109,
  permuteSecond: 419,
  memStride: 4,
  aliasedMemory: false,
  numBehaviors: 2
};

// The two kinds of non-atomic accesses tested.
//  rw: read -> barrier -> write
//  wr: write -> barrier -> read
//  ww: write -> barrier -> write


// Test the non-atomic memory types.
const kMemTypes = [MemoryType.NonAtomicStorageClass, MemoryType.NonAtomicWorkgroupClass];

const storageMemoryBarrierStoreLoadTestCode = `
  test_locations.value[x_0] = 1;
  workgroupBarrier();
  let r0 = u32(test_locations.value[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
`;

const workgroupMemoryBarrierStoreLoadTestCode = `
  wg_test_locations[x_0] = 1;
  workgroupBarrier();
  let r0 = u32(wg_test_locations[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
`;

const storageMemoryBarrierLoadStoreTestCode = `
  let r0 = u32(test_locations.value[x_0]);
  workgroupBarrier();
  test_locations.value[x_1] = 1;
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const workgroupMemoryBarrierLoadStoreTestCode = `
  let r0 = u32(wg_test_locations[x_0]);
  workgroupBarrier();
  wg_test_locations[x_1] = 1;
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const storageMemoryBarrierStoreStoreTestCode = `
  test_locations.value[x_0] = 1;
  storageBarrier();
  test_locations.value[x_1] = 2;
`;

const workgroupMemoryBarrierStoreStoreTestCode = `
  wg_test_locations[x_0] = 1;
  workgroupBarrier();
  wg_test_locations[x_1] = 2;
  workgroupBarrier();
  test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1] = wg_test_locations[x_1];
`;

function getTestCode(p) {
  switch (p.accessPair) {
    case 'rw':
      return p.memType === MemoryType.NonAtomicStorageClass ?
      storageMemoryBarrierLoadStoreTestCode :
      workgroupMemoryBarrierLoadStoreTestCode;
    case 'wr':
      return p.memType === MemoryType.NonAtomicStorageClass ?
      storageMemoryBarrierStoreLoadTestCode :
      workgroupMemoryBarrierStoreLoadTestCode;
    case 'ww':
      return p.memType === MemoryType.NonAtomicStorageClass ?
      storageMemoryBarrierStoreStoreTestCode :
      workgroupMemoryBarrierStoreStoreTestCode;
  }
}

g.test('workgroup_barrier_store_load').
desc(
  `Checks whether the workgroup barrier properly synchronizes a non-atomic write and read on
    separate threads in the same workgroup. Within a workgroup, the barrier should force an invocation
    after the barrier to read a write from an invocation before the barrier.
    `
).
params((u) =>
u.
combine('accessValueType', kAccessValueTypes).
combine('memType', kMemTypes).
combine('accessPair', ['wr'])
).
beforeAllSubcases((t) => {
  if (t.params.accessValueType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const resultCode = `
      if (r0 == 1u) {
        atomicAdd(&test_results.seq, 1u);
      } else if (r0 == 0u) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  const testShader = buildTestShader(
    getTestCode(t.params),
    t.params.memType,
    TestType.IntraWorkgroup
  );
  const resultShader = buildResultShader(
    resultCode,
    TestType.IntraWorkgroup,
    ResultType.TwoBehavior
  );
  const memModelTester = new MemoryModelTester(
    t,
    memoryModelTestParams,
    testShader,
    resultShader,
    t.params.accessValueType
  );
  await memModelTester.run(15, 1);
});

g.test('workgroup_barrier_load_store').
desc(
  `Checks whether the workgroup barrier properly synchronizes a non-atomic write and read on
    separate threads in the same workgroup. Within a workgroup, the barrier should force an invocation
    before the barrier to not read the write from an invocation after the barrier.
    `
).
params((u) =>
u.
combine('accessValueType', kAccessValueTypes).
combine('memType', kMemTypes).
combine('accessPair', ['rw'])
).
beforeAllSubcases((t) => {
  if (t.params.accessValueType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const resultCode = `
      if (r0 == 0u) {
        atomicAdd(&test_results.seq, 1u);
      } else if (r0 == 1u) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  const testShader = buildTestShader(
    getTestCode(t.params),
    t.params.memType,
    TestType.IntraWorkgroup
  );
  const resultShader = buildResultShader(
    resultCode,
    TestType.IntraWorkgroup,
    ResultType.TwoBehavior
  );
  const memModelTester = new MemoryModelTester(
    t,
    memoryModelTestParams,
    testShader,
    resultShader,
    t.params.accessValueType
  );
  await memModelTester.run(12, 1);
});

g.test('workgroup_barrier_store_store').
desc(
  `Checks whether the workgroup barrier properly synchronizes non-atomic writes on
    separate threads in the same workgroup. Within a workgroup, the barrier should force the value in memory
    to be the result of the write after the barrier, not the write before.
    `
).
params((u) =>
u.
combine('accessValueType', kAccessValueTypes).
combine('memType', kMemTypes).
combine('accessPair', ['ww'])
).
beforeAllSubcases((t) => {
  if (t.params.accessValueType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const resultCode = `
      if (mem_x_0 == 2u) {
        atomicAdd(&test_results.seq, 1u);
      } else if (mem_x_0 == 1u) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  const testShader = buildTestShader(
    getTestCode(t.params),
    t.params.memType,
    TestType.IntraWorkgroup
  );
  const resultShader = buildResultShader(
    resultCode,
    TestType.IntraWorkgroup,
    ResultType.TwoBehavior
  );
  const memModelTester = new MemoryModelTester(
    t,
    memoryModelTestParams,
    testShader,
    resultShader,
    t.params.accessValueType
  );
  await memModelTester.run(10, 1);
});