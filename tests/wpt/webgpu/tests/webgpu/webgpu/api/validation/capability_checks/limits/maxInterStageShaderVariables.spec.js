/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { keysOf } from '../../../../../common/util/data_tables.js';import { hasFeature, range } from '../../../../../common/util/util.js';

import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';

const kFragmentInputTypes = {
  front_facing: 'bool',
  sample_index: 'u32',
  sample_mask: 'u32',
  primitive_index: 'u32',
  subgroup_invocation_id: 'u32',
  subgroup_size: 'u32'
};

const kFragmentInputs = keysOf(kFragmentInputTypes);

const kItemsThatCountAgainstLimit = ['point-list', ...kFragmentInputs];

const kExtraItems = [
'sample_mask_out' // special - see below
];

/**
 * Generates every combination of size elements from array
 * combinations([a, b, c], 2) generates [a, b], [a, c], [b, c]
 */
function* combinations(
arr,
size,
start = 0,
path = [])
{
  if (path.length === size) {
    yield [...path];
    return;
  }

  for (let i = start; i < arr.length; i++) {
    path.push(arr[i]);
    yield* combinations(arr, size, i + 1, path);
    path.pop();
  }
}

const kTestItems = [...kItemsThatCountAgainstLimit, ...kExtraItems];
const kTestItemCombinations = [
[], // no builtins case
...combinations(kTestItems, 1), // one builtin
...combinations(kTestItems, 2), // 2 builtins
...combinations(kTestItems, 3), // 3 builtins
kTestItems // all builtins case
];

const requiresSubgroupsFeature = (items) =>
items.has('subgroup_invocation_id') || items.has('subgroup_size');

function getPipelineDescriptor(
t,
device,
testValue,
items)
{
  const vertexOutputDeductions = items.has('point-list') ? 1 : 0;
  const usedFragInputs = [...items.values()].filter((p) => p in kFragmentInputTypes);
  const fragmentInputDeductions = usedFragInputs.
  map((p) => p ? 1 : 0).
  reduce((acc, p) => acc + p, 0);

  t.debug(() => `device features: ${[...device.features].join(', ')}`);

  const numVertexOutputVariables = testValue - vertexOutputDeductions;
  const numFragmentInputVariables = testValue - fragmentInputDeductions;
  const numInterStageVariables = Math.min(numVertexOutputVariables, numFragmentInputVariables);

  const maxVertexOutputVariables =
  device.limits.maxInterStageShaderVariables - vertexOutputDeductions;
  const maxFragmentInputVariables =
  device.limits.maxInterStageShaderVariables - fragmentInputDeductions;
  const maxInterStageVariables = Math.min(maxVertexOutputVariables, maxFragmentInputVariables);

  const fragInputs = usedFragInputs.
  map(
    (input, i) =>
    `      @builtin(${input}) i_${i}: ${
    kFragmentInputTypes[input]
    },`
  ).
  join('\n');

  const varyings = `${range(
    numInterStageVariables,
    (i) => `      @location(${i}) v4_${i}: vec4f,`
  ).join('\n')}`;

  const code = `
    // test value                        : ${testValue}
    // maxInterStageShaderVariables      : ${device.limits.maxInterStageShaderVariables}
    // num variables in vertex shader    : ${numVertexOutputVariables}${
  items.has('point-list') ? ' + point-list' : ''
  }
    // num variables in fragment shader  : ${numFragmentInputVariables} + ${usedFragInputs.join(
    ' + '
  )}
    // maxInterStageVariables:           : ${maxInterStageVariables}
    // num used inter stage variables    : ${numInterStageVariables}

    ${items.has('primitive_index') ? 'enable primitive_index;' : ''}
    ${requiresSubgroupsFeature(items) ? 'enable subgroups;' : ''}

    struct VSOut {
      @builtin(position) p: vec4f,
${varyings}
    }
    struct FSIn {
${fragInputs}
${varyings}
    }

    struct FSOut {
      @location(0) color: vec4f,
      ${items.has('sample_mask_out') ? '@builtin(sample_mask) sampleMask: u32,' : ''}
    }

    @vertex fn vs() -> VSOut {
      var o: VSOut;
      o.p = vec4f(0);
      return o;
    }

    @fragment fn fs(i: FSIn) -> FSOut {
      var o: FSOut;

      o.color = vec4f(0);
      return o;
    }
  `;
  t.debug(code);
  const module = device.createShaderModule({ code });
  const pipelineDescriptor = {
    layout: 'auto',
    primitive: {
      topology: items.has('point-list') ? 'point-list' : 'triangle-list'
    },
    vertex: {
      module,
      entryPoint: 'vs'
    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets: [
      {
        format: 'rgba8unorm'
      }]

    }
  };
  return pipelineDescriptor;
}

const limit = 'maxInterStageShaderVariables';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createRenderPipeline,at_over').
desc(
  `
Test using at and over ${limit} limit in createRenderPipeline(Async)

Note: We test combinations to make sure each entry is counted separately.
and that implementations don't accidentally add only 1 to the count when
2 or more builtins are used. We also include sample_mask as an output
to make sure it does not count against the limit since it has the same
name as sample_mask as an input.
  `
).
params(
  kMaximumLimitBaseParams.combine('async', [false, true]).combine('items', kTestItemCombinations)
).
fn(async (t) => {
  const { limitTest, testValueName, async, items: itemsAsArray } = t.params;
  const items = new Set(itemsAsArray);

  if (t.isCompatibility) {
    t.skipIf(
      items.has('sample_mask') || items.has('sample_mask_out'),
      'sample_mask not supported in compatibility mode'
    );
    t.skipIf(items.has('sample_index'), 'sample_index not supported in compatibility mode');
  }

  const features = [];

  if (items.has('primitive_index')) {
    if (hasFeature(t.adapter.features, 'primitive-index')) {
      features.push('primitive-index');
    } else {
      t.skip('primitive_index requires primitive-index feature');
    }
  }

  if (requiresSubgroupsFeature(items)) {
    if (hasFeature(t.adapter.features, 'subgroups')) {
      features.push('subgroups');
    } else {
      t.skip('subgroup_invocation_id or subgroup_size requires subgroups feature');
    }
  }

  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const pipelineDescriptor = getPipelineDescriptor(t, device, testValue, items);

      await t.testCreateRenderPipeline(pipelineDescriptor, async, shouldError);
    },
    undefined,
    features
  );
});