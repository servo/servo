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

// The three kinds of non-atomic accesses tested.
//  rw: read -> barrier -> write
//  wr: write -> barrier -> read
//  ww: write -> barrier -> write


// Test the non-atomic memory types.
const kMemTypes = [
MemoryType.NonAtomicStorageClass,
MemoryType.NonAtomicWorkgroupClass,
MemoryType.NonAtomicTextureClass];


const storageMemoryBarrierStoreLoadTestCode = `
  test_locations.value[x_0] = 1;
  storageBarrier();
  let r0 = u32(test_locations.value[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
`;

const textureMemoryBarrierStoreLoadTestCode = `
  textureStore(texture_locations, indexToCoord(x_0), vec4u(1));
  textureBarrier();
  let r0 = textureLoad(texture_locations, indexToCoord(x_1)).x;
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
`;

const workgroupMemoryBarrierStoreLoadTestCode = `
  wg_test_locations[x_0] = 1;
  workgroupBarrier();
  let r0 = u32(wg_test_locations[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
`;

const workgroupUniformLoadMemoryBarrierStoreLoadTestCode = `
  wg_test_locations[x_0] = 1;
  _ = workgroupUniformLoad(&placeholder_wg_var);
  let r0 = u32(wg_test_locations[x_1]);
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_1].r0, r0);
`;

const storageMemoryBarrierLoadStoreTestCode = `
  let r0 = u32(test_locations.value[x_0]);
  storageBarrier();
  test_locations.value[x_1] = 1;
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const textureMemoryBarrierLoadStoreTestCode = `
  let r0 = textureLoad(texture_locations, indexToCoord(x_0)).x;
  textureBarrier();
  textureStore(texture_locations, indexToCoord(x_1), vec4u(1));
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const workgroupMemoryBarrierLoadStoreTestCode = `
  let r0 = u32(wg_test_locations[x_0]);
  workgroupBarrier();
  wg_test_locations[x_1] = 1;
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const workgroupUniformLoadMemoryBarrierLoadStoreTestCode = `
  let r0 = u32(wg_test_locations[x_0]);
  _ = workgroupUniformLoad(&placeholder_wg_var);
  wg_test_locations[x_1] = 1;
  atomicStore(&results.value[shuffled_workgroup * workgroupXSize + id_0].r0, r0);
`;

const storageMemoryBarrierStoreStoreTestCode = `
  test_locations.value[x_0] = 1;
  storageBarrier();
  test_locations.value[x_1] = 2;
`;

const textureMemoryBarrierStoreStoreTestCode = `
  textureStore(texture_locations, indexToCoord(x_0), vec4u(1));
  textureBarrier();
  textureStore(texture_locations, indexToCoord(x_1), vec4u(2));
  textureBarrier();
  test_locations.value[x_1] = textureLoad(texture_locations, indexToCoord(x_1)).x;
`;

const workgroupMemoryBarrierStoreStoreTestCode = `
  wg_test_locations[x_0] = 1;
  workgroupBarrier();
  wg_test_locations[x_1] = 2;
  workgroupBarrier();
  test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1] = wg_test_locations[x_1];
`;

const workgroupUniformLoadMemoryBarrierStoreStoreTestCode = `
  wg_test_locations[x_0] = 1;
  _ = workgroupUniformLoad(&placeholder_wg_var);
  wg_test_locations[x_1] = 2;
  _ = workgroupUniformLoad(&placeholder_wg_var);
  test_locations.value[shuffled_workgroup * workgroupXSize * stress_params.mem_stride * 2u + x_1] = wg_test_locations[x_1];
`;

function getTestCode(p)



{
  switch (p.accessPair) {
    case 'rw':{
        switch (p.memType) {
          case MemoryType.NonAtomicStorageClass:
            return storageMemoryBarrierLoadStoreTestCode;
          case MemoryType.NonAtomicTextureClass:
            return textureMemoryBarrierLoadStoreTestCode;
          default:
            return p.normalBarrier ?
            workgroupMemoryBarrierLoadStoreTestCode :
            workgroupUniformLoadMemoryBarrierLoadStoreTestCode;
        }
      }
    case 'wr':{
        switch (p.memType) {
          case MemoryType.NonAtomicStorageClass:
            return storageMemoryBarrierStoreLoadTestCode;
          case MemoryType.NonAtomicTextureClass:
            return textureMemoryBarrierStoreLoadTestCode;
          default:
            return p.normalBarrier ?
            workgroupMemoryBarrierStoreLoadTestCode :
            workgroupUniformLoadMemoryBarrierStoreLoadTestCode;
        }
      }
    case 'ww':{
        switch (p.memType) {
          case MemoryType.NonAtomicStorageClass:
            return storageMemoryBarrierStoreStoreTestCode;
          case MemoryType.NonAtomicTextureClass:
            return textureMemoryBarrierStoreStoreTestCode;
          default:
            return p.normalBarrier ?
            workgroupMemoryBarrierStoreStoreTestCode :
            workgroupUniformLoadMemoryBarrierStoreStoreTestCode;
        }
      }
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
combine('accessPair', ['wr']).
combine('normalBarrier', [true, false])
).
beforeAllSubcases((t) => {
  if (t.params.accessValueType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
  t.skipIf(
    !t.params.normalBarrier && t.params.memType !== MemoryType.NonAtomicWorkgroupClass,
    'workgroupUniformLoad does not have storage memory semantics'
  );
  t.skipIf(
    t.params.memType === MemoryType.NonAtomicTextureClass && t.params.accessValueType === 'f16',
    'textures do not support f16 access'
  );
}).
fn(async (t) => {
  t.skipIf(
    t.params.memType === MemoryType.NonAtomicTextureClass &&
    !t.hasLanguageFeature('readonly_and_readwrite_storage_textures'),
    'requires RW storage textures feature'
  );

  const resultCode = `
      if (r0 == 1u) {
        atomicAdd(&test_results.seq, 1u);
      } else if (r0 == 0u) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  let testShader = buildTestShader(
    getTestCode(t.params),
    t.params.memType,
    TestType.IntraWorkgroup
  );
  if (!t.params.normalBarrier) {
    testShader += '\nvar<workgroup> placeholder_wg_var : u32;\n';
  }
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
    t.params.accessValueType,
    t.params.memType === MemoryType.NonAtomicTextureClass
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
combine('accessPair', ['rw']).
combine('normalBarrier', [true, false])
).
beforeAllSubcases((t) => {
  if (t.params.accessValueType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
  t.skipIf(
    !t.params.normalBarrier && t.params.memType !== MemoryType.NonAtomicWorkgroupClass,
    'workgroupUniformLoad does not have storage memory semantics'
  );
  t.skipIf(
    t.params.memType === MemoryType.NonAtomicTextureClass && t.params.accessValueType === 'f16',
    'textures do not support f16 access'
  );
}).
fn(async (t) => {
  t.skipIf(
    t.params.memType === MemoryType.NonAtomicTextureClass &&
    !t.hasLanguageFeature('readonly_and_readwrite_storage_textures'),
    'requires RW storage textures feature'
  );

  const resultCode = `
      if (r0 == 0u) {
        atomicAdd(&test_results.seq, 1u);
      } else if (r0 == 1u) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  let testShader = buildTestShader(
    getTestCode(t.params),
    t.params.memType,
    TestType.IntraWorkgroup
  );
  if (!t.params.normalBarrier) {
    testShader += '\nvar<workgroup> placeholder_wg_var : u32;\n';
  }
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
    t.params.accessValueType,
    t.params.memType === MemoryType.NonAtomicTextureClass
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
combine('accessPair', ['ww']).
combine('normalBarrier', [true, false])
).
beforeAllSubcases((t) => {
  if (t.params.accessValueType === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
  t.skipIf(
    !t.params.normalBarrier && t.params.memType !== MemoryType.NonAtomicWorkgroupClass,
    'workgroupUniformLoad does not have storage memory semantics'
  );
  t.skipIf(
    t.params.memType === MemoryType.NonAtomicTextureClass && t.params.accessValueType === 'f16',
    'textures do not support f16 access'
  );
}).
fn(async (t) => {
  t.skipIf(
    t.params.memType === MemoryType.NonAtomicTextureClass &&
    !t.hasLanguageFeature('readonly_and_readwrite_storage_textures'),
    'requires RW storage textures feature'
  );

  const resultCode = `
      if (mem_x_0 == 2u) {
        atomicAdd(&test_results.seq, 1u);
      } else if (mem_x_0 == 1u) {
        atomicAdd(&test_results.weak, 1u);
      }
    `;
  let testShader = buildTestShader(
    getTestCode(t.params),
    t.params.memType,
    TestType.IntraWorkgroup
  );
  if (!t.params.normalBarrier) {
    testShader += '\nvar<workgroup> placeholder_wg_var : u32;\n';
  }
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
    t.params.accessValueType,
    t.params.memType === MemoryType.NonAtomicTextureClass
  );
  await memModelTester.run(10, 1);
});