/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for subgroupBallot

Note: There is a lack of portability for non-uniform execution so these tests
restrict themselves to uniform control flow or returning early.
Note: There is no guaranteed mapping between subgroup_invocation_id and
local_invocation_index. Tests should avoid assuming there is.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { iterRange } from '../../../../../../common/util/util.js';
import { GPUTest } from '../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

// 128 is the maximum possible subgroup size.
const kInvocations = 128;

function getMask(size) {
  return (1n << BigInt(size)) - 1n;
}

function checkBallots(
data,
subgroupSize,
filter,
expect,
allActive)
{
  for (let i = 0; i < kInvocations; i++) {
    const idx = i * 4;
    let actual = 0n;
    for (let j = 0; j < 4; j++) {
      actual |= BigInt(data[idx + j]) << BigInt(32 * j);
    }
    let expectedResult = expect(subgroupSize);
    const subgroupId = i % subgroupSize;
    if (!allActive && !filter(subgroupId, subgroupSize)) {
      expectedResult = 0n;
    }
    if (expectedResult !== actual) {
      return new Error(
        `Invocation ${i}, subgroup inv id ${i % subgroupSize}, size ${subgroupSize}
- expected: ${expectedResult.toString(16)}
-      got: ${actual.toString(16)}`
      );
    }
  }

  return undefined;
}

async function runTest(
t,
wgsl,
filter,
expect,
allActive)
{
  const sizeBuffer = t.makeBufferWithContents(
    new Uint32Array([0]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(sizeBuffer);

  const outputNumInts = kInvocations * 4;
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(outputNumInts, (x) => 0)]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  );
  t.trackForCleanup(outputBuffer);

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: wgsl
      }),
      entryPoint: 'main'
    }
  });
  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: sizeBuffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: outputBuffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const sizeReadback = await t.readGPUBufferRangeTyped(sizeBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: 1,
    method: 'copy'
  });
  const subgroupSize = sizeReadback.data[0];

  const outputReadback = await t.readGPUBufferRangeTyped(outputBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: outputNumInts,
    method: 'copy'
  });
  const output = outputReadback.data;

  t.expectOK(checkBallots(output, subgroupSize, filter, expect, allActive));
}

const kCases = {
  every_even: {
    cond: `id % 2 == 0`,
    filter: (id, size) => {
      return id % 2 === 0;
    },
    expect: (subgroupSize) => {
      const base = BigInt('0x55555555555555555555555555555555');
      const mask = getMask(subgroupSize);
      return base & mask;
    }
  },
  every_odd: {
    cond: `id % 2 == 1`,
    filter: (id, size) => {
      return id % 2 === 1;
    },
    expect: (subgroupSize) => {
      const base = BigInt('0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA');
      const mask = getMask(subgroupSize);
      return base & mask;
    }
  },
  lower_half: {
    cond: `id < subgroupSize / 2`,
    filter: (id, size) => {
      return id < Math.floor(size / 2);
    },
    expect: (size) => {
      return getMask(Math.floor(size / 2));
    }
  },
  upper_half: {
    cond: `id >= subgroupSize / 2`,
    filter: (id, size) => {
      return id >= Math.floor(size / 2);
    },
    expect: (size) => {
      return getMask(Math.floor(size / 2)) << BigInt(Math.floor(size / 2));
    }
  },
  first_two: {
    cond: `id == 0 || id == 1`,
    filter: (id) => {
      return id === 0 || id === 1;
    },
    expect: (size) => {
      return getMask(2);
    }
  }
};

g.test('compute,split').
desc('Tests ballot in a split subgroup').
params((u) => u.combine('case', keysOf(kCases))).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const testcase = kCases[t.params.case];
  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage, read_write> size : u32;

@group(0) @binding(1)
var<storage, read_write> output : array<vec4u>;

@compute @workgroup_size(${kInvocations})
fn main(@builtin(subgroup_size) subgroupSize : u32,
        @builtin(subgroup_invocation_id) id : u32,
        @builtin(local_invocation_index) lid : u32) {
  if (lid == 0) {
    size = subgroupSize;
  }
  if ${testcase.cond} {
    output[lid] = subgroupBallot(true);
  } else {
    return;
  }
}`;

  await runTest(t, wgsl, testcase.filter, testcase.expect, false);
});

g.test('fragment,split').unimplemented();

g.test('predicate').
desc('Tests the predicate parameter').
params((u) => u.combine('case', keysOf(kCases))).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const testcase = kCases[t.params.case];
  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage, read_write> size : u32;

@group(0) @binding(1)
var<storage, read_write> output : array<vec4u>;

@compute @workgroup_size(${kInvocations})
fn main(@builtin(subgroup_size) subgroupSize : u32,
        @builtin(subgroup_invocation_id) id : u32,
        @builtin(local_invocation_index) lid : u32) {
  if (lid == 0) {
    size = subgroupSize;
  }
  let cond = ${testcase.cond};
  let b = subgroupBallot(cond);
  output[lid] = b;
}`;

  await runTest(t, wgsl, testcase.filter, testcase.expect, true);
});

const kBothCases = {
  empty: {
    cond: `id < subgroupSize / 2`,
    pred: `id >= subgroupSize / 2`,
    filter: (id, size) => {
      return id < Math.floor(size / 2);
    },
    expect: (size) => {
      return 0n;
    }
  },
  full: {
    cond: `id < 128`,
    pred: `lid < 128`,
    filter: (id, size) => {
      return true;
    },
    expect: (size) => {
      return getMask(size);
    }
  },
  one_in_four: {
    cond: `id % 2 == 0`,
    pred: `id % 4 == 0`,
    filter: (id, size) => {
      return id % 2 === 0;
    },
    expect: (size) => {
      const base = BigInt('0x11111111111111111111111111111111');
      const mask = getMask(size);
      return base & mask;
    }
  },
  middle_half: {
    cond: `id >= subgroupSize / 4`,
    pred: `id < 3 * (subgroupSize / 4)`,
    filter: (id, size) => {
      return id >= Math.floor(size / 4);
    },
    expect: (size) => {
      return getMask(Math.floor(size / 2)) << BigInt(Math.floor(size / 4));
    }
  },
  middle_half_every_other: {
    cond: `(id >= subgroupSize / 4) && (id < 3 * (subgroupSize / 4))`,
    pred: `id % 2 == 0`,
    filter: (id, size) => {
      return id >= Math.floor(size / 4) && id < 3 * Math.floor(size / 4);
    },
    expect: (size) => {
      const base = BigInt('0x55555555555555555555555555555555');
      const mask = getMask(Math.floor(size / 2)) << BigInt(Math.floor(size / 4));
      return base & mask;
    }
  }
};

g.test('predicate_and_control_flow').
desc('Test dynamic predicate and control flow together').
params((u) => u.combine('case', keysOf(kBothCases))).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('subgroups');
}).
fn(async (t) => {
  const testcase = kBothCases[t.params.case];
  const wgsl = `
enable subgroups;

@group(0) @binding(0)
var<storage, read_write> size : u32;

@group(0) @binding(1)
var<storage, read_write> output : array<vec4u>;

@compute @workgroup_size(${kInvocations})
fn main(@builtin(subgroup_size) subgroupSize : u32,
        @builtin(subgroup_invocation_id) id : u32,
        @builtin(local_invocation_index) lid : u32) {
  if (lid == 0) {
    size = subgroupSize;
  }
  if ${testcase.cond} {
    output[lid] = subgroupBallot(${testcase.pred});
  } else {
    return;
  }
}`;

  await runTest(t, wgsl, testcase.filter, testcase.expect, false);
});

g.test('fragment').unimplemented();