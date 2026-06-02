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
import { iterRange, assert } from '../../../../../../common/util/util.js';
import { getBlockInfoForTextureFormat } from '../../../../../format_info.js';

import * as ttu from '../../../../../texture_test_utils.js';
import { align } from '../../../../../util/math.js';

import { SubgroupTest, kFramebufferSizes, getUintsPerFramebuffer } from './subgroup_util.js';

export const g = makeTestGroup(SubgroupTest);

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
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kCases[t.params.case];
  const wgsl = `
enable subgroups;

diagnostic(off, subgroup_uniformity);
diagnostic(off, subgroup_branching);

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
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kCases[t.params.case];
  const wgsl = `
enable subgroups;

diagnostic(off, subgroup_branching);

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
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const testcase = kBothCases[t.params.case];
  const wgsl = `
enable subgroups;

diagnostic(off, subgroup_branching);
diagnostic(off, subgroup_uniformity);

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

// Filters should always skip the last row and column.
const kFragmentPredicates = {
  odd_row: {
    cond: `u32(pos.y) % 2 == 1`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return row % 2 === 1;
    }
  },
  even_row: {
    cond: `u32(pos.y) % 2 == 0`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return row % 2 === 0;
    }
  },
  odd_col: {
    cond: `u32(pos.x) % 2 == 1`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return col % 2 === 1;
    }
  },
  even_col: {
    cond: `u32(pos.x) % 2 == 0`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return col % 2 === 0;
    }
  },
  odd_id: {
    cond: `id % 2 == 1`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return id % 2 === 1;
    }
  },
  even_id: {
    cond: `id % 2 == 0`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return id % 2 === 0;
    }
  },
  upper_half: {
    cond: `id > subgroupSize / 2`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return id > Math.floor(size / 2);
    }
  },
  lower_half: {
    cond: `id < subgroupSize / 2`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return id < Math.floor(size / 2);
    }
  },
  first_two_or_diagonal: {
    cond: `id == 0 || id == 1 || u32(pos.x) == u32(pos.y)`,
    filter: (row, col, width, height, id, size) => {
      if (row === height - 1 || col === width - 1) {
        return false;
      }
      return id === 0 || id === 1 || row === col;
    }
  }
};

/**
 * Checks the result of subgroupBallot in fragment shaders.
 *
 * Extra bits are allowed in ballots due to helpers, but results must be consistent
 * among invocations known to be good.
 * @param ballots Framebuffer of ballot results
 * @param metadata Framebuffer of metadata
 *                 * component 0 is subgroup_invocation_id
 *                 * component 1 is subgroup_size
 *                 * component 2 is a unique, generated subgroup id
 * @param format The framebuffer format
 * @param width The framebuffer width
 * @param height The framebuffer height
 * @param filter A functor that returns true if the invocation should be included in the ballot
 */
function checkFragmentBallots(
ballots,
metadata,
format,
width,
height,
filter)







{
  if (width < 3 || height < 3) {
    return new Error(
      `Insufficient framebuffer size [${width}w x ${height}h]. Minimum is [3w x 3h].`
    );
  }

  const { uintsPerRow, uintsPerTexel } = getUintsPerFramebuffer(format, width, height);

  const coordToIndex = (row, col) => {
    return uintsPerRow * row + col * uintsPerTexel;
  };

  const mapping = new Map();

  // Iteration skips last row and column to avoid helper invocations because it is not
  // guaranteed whether or not they participate in the subgroup operation.
  for (let row = 0; row < height - 1; row++) {
    for (let col = 0; col < width - 1; col++) {
      const offset = coordToIndex(row, col);

      const id = metadata[offset];
      const subgroupSize = metadata[offset + 1];
      const subgroupId = metadata[offset + 2];

      let ballot = BigInt(ballots[offset]);
      ballot |= BigInt(ballots[offset + 1]) << 32n;
      ballot |= BigInt(ballots[offset + 2]) << 64n;
      ballot |= BigInt(ballots[offset + 3]) << 96n;

      const expectBit = filter(row, col, width, height, id, subgroupSize) ? 1n : 0n;
      const gotBit = ballot >> BigInt(id) & 1n;

      if (expectBit !== gotBit) {
        return new Error(`Row ${row}, col ${col}: incorrect ballot bit ${id}:
- expected: ${expectBit.toString(10)}
-      got: ${gotBit.toString(10)}`);
      }

      const expected = mapping.get(subgroupId);
      if (expected === undefined) {
        mapping.set(subgroupId, ballot);
      } else {
        if (expected !== ballot) {
          return new Error(`Row ${row} col ${col}: ballot mismatch:
- expected: ${expected.toString(16)}
-      got: ${ballot.toString(16)}`);
        }
      }
    }
  }

  return undefined;
}

g.test('fragment').
desc('Tests subgroupBallot in fragment shaders').
params((u) =>
u.
combine('predicate', keysOf(kFragmentPredicates)).
beginSubcases().
combine('size', kFramebufferSizes).
combineWithParams([{ format: 'rgba32uint' }])
).
fn(async (t) => {
  t.skipIfDeviceDoesNotHaveFeature('subgroups');
  const width = t.params.size[0];
  const height = t.params.size[1];
  const testcase = kFragmentPredicates[t.params.predicate];

  const fsShader = `
enable subgroups;

struct FSOutput {
  @location(0) ballot : vec4u,
  @location(1) metadata : vec4u,
}

@fragment
fn main(
  @builtin(position) pos : vec4f,
  @builtin(subgroup_size) subgroupSize : u32,
  @builtin(subgroup_invocation_id) id : u32,
) -> FSOutput {
  let linear = u32(pos.x) + u32(pos.y) * ${width};
  let subgroup_id = subgroupBroadcastFirst(linear + 1);

  // Filter out possible helper invocations.
  let x_in_range = u32(pos.x) < (${width} - 1);
  let y_in_range = u32(pos.y) < (${height} - 1);
  let in_range = x_in_range && y_in_range;

  let cond = ${testcase.cond};
  let ballot = subgroupBallot(in_range && cond);

  var out : FSOutput;
  out.ballot = ballot;
  out.metadata = vec4u(id, subgroupSize, subgroup_id, 0);
  return out;
}`;

  const vsShader = `
@vertex
fn vsMain(@builtin(vertex_index) index : u32) -> @builtin(position) vec4f {
  const vertices = array(
    vec2(-2, 4), vec2(-2, -4), vec2(2, 0),
  );
  return vec4f(vec2f(vertices[index]), 0, 1);
}`;

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({ code: vsShader })
    },
    fragment: {
      module: t.device.createShaderModule({ code: fsShader }),
      targets: [{ format: t.params.format }, { format: t.params.format }]
    },
    primitive: {
      topology: 'triangle-list'
    }
  });

  const { blockWidth, blockHeight, bytesPerBlock } = getBlockInfoForTextureFormat(
    t.params.format
  );
  assert(bytesPerBlock !== undefined);

  const blocksPerRow = width / blockWidth;
  const blocksPerColumn = height / blockHeight;
  // 256 minimum arises from image copy requirements.
  const bytesPerRow = align(blocksPerRow * (bytesPerBlock ?? 1), 256);
  const byteLength = bytesPerRow * blocksPerColumn;
  const uintLength = byteLength / 4;

  const ballotFB = t.createTextureTracked({
    size: [width, height],
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING,
    format: t.params.format
  });

  const metadataFB = t.createTextureTracked({
    size: [width, height],
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING,
    format: t.params.format
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: ballotFB.createView(),
      loadOp: 'clear',
      storeOp: 'store'
    },
    {
      view: metadataFB.createView(),
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const ballotBuffer = ttu.copyWholeTextureToNewBufferSimple(t, ballotFB, 0);
  const ballotReadback = await t.readGPUBufferRangeTyped(ballotBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: uintLength,
    method: 'copy'
  });
  const ballots = ballotReadback.data;

  const metadataBuffer = ttu.copyWholeTextureToNewBufferSimple(t, metadataFB, 0);
  const metadataReadback = await t.readGPUBufferRangeTyped(metadataBuffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: uintLength,
    method: 'copy'
  });
  const metadata = metadataReadback.data;

  t.expectOK(
    checkFragmentBallots(ballots, metadata, t.params.format, width, height, testcase.filter)
  );
});