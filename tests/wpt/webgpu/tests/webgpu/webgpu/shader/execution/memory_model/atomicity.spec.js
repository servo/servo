/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Tests for the atomicity of atomic read-modify-write instructions.`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

import {
  MemoryModelTester,
  buildTestShader,
  TestType,
  buildResultShader,
  ResultType,
  MemoryType,
} from './memory_model_setup.js';

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
  numBehaviors: 4,
};

const storageMemoryTestCode = `
  let r0 = atomicAdd(&test_locations.value[x_0], 0u);
  atomicStore(&test_locations.value[x_1], 2u);
  atomicStore(&results.value[id_0].r0, r0);
`;

const workgroupMemoryTestCode = `
  let r0 = atomicAdd(&wg_test_locations[x_0], 0u);
  atomicStore(&wg_test_locations[x_1], 2u);
  workgroupBarrier();
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
  atomicStore(&test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1], atomicLoad(&wg_test_locations[x_1]));
`;

const resultCode = `
  if ((r0 == 0u && mem_x_0 == 2u)) {
    atomicAdd(&test_results.seq0, 1u);
  } else if ((r0 == 2u && mem_x_0 == 1u)) {
    atomicAdd(&test_results.seq1, 1u);
  } else if ((r0 == 0u && mem_x_0 == 1u)) {
    atomicAdd(&test_results.weak, 1u);
  }
`;

g.test('atomicity')
  .desc(
    `Checks whether a store on one thread can interrupt an atomic RMW on a second thread. If the read returned by
    the RMW instruction is the initial value of memory (0), but the final value in memory is 1, then the atomic write
    in the second thread occurred in between the read and the write of the RMW.
    `
  )
  .paramsSimple([
    {
      memType: MemoryType.AtomicStorageClass,
      testType: TestType.InterWorkgroup,
      _testCode: storageMemoryTestCode,
    },
    {
      memType: MemoryType.AtomicStorageClass,
      testType: TestType.IntraWorkgroup,
      _testCode: storageMemoryTestCode,
    },
    {
      memType: MemoryType.AtomicWorkgroupClass,
      testType: TestType.IntraWorkgroup,
      _testCode: workgroupMemoryTestCode,
    },
  ])
  .fn(async t => {
    const testShader = buildTestShader(t.params._testCode, t.params.memType, t.params.testType);
    const resultShader = buildResultShader(resultCode, t.params.testType, ResultType.FourBehavior);
    const memModelTester = new MemoryModelTester(
      t,
      memoryModelTestParams,
      testShader,
      resultShader
    );

    await memModelTester.run(10, 3);
  });
