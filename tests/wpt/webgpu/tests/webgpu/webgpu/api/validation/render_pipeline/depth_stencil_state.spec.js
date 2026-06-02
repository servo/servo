/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
This test dedicatedly tests validation of GPUDepthStencilState of createRenderPipeline.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { unreachable } from '../../../../common/util/util.js';
import {
  kCompareFunctions,
  kPrimitiveTopology,
  kStencilOperations } from
'../../../capability_info.js';
import {
  kAllTextureFormats,
  kDepthStencilFormats,
  isDepthOrStencilTextureFormat,
  isDepthTextureFormat,
  isStencilTextureFormat } from
'../../../format_info.js';
import { getFragmentShaderCodeWithOutput } from '../../../util/shader.js';
import * as vtu from '../validation_test_utils.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

g.test('format').
desc(`The texture format in depthStencilState must be a depth/stencil format.`).
params((u) =>
u //
.combine('isAsync', [false, true]).
combine('format', kAllTextureFormats)
).
fn((t) => {
  const { isAsync, format } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const descriptor = t.getDescriptor({
    depthStencil: { format, depthWriteEnabled: false, depthCompare: 'always' }
  });

  vtu.doCreateRenderPipelineTest(t, isAsync, isDepthOrStencilTextureFormat(format), descriptor);
});

g.test('depthCompare_optional').
desc(
  `The depthCompare in depthStencilState is optional for stencil-only formats but
    required for formats with a depth if depthCompare is not used for anything.`
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', kDepthStencilFormats).
beginSubcases().
combine('depthCompare', ['always', undefined]).
combine('depthWriteEnabled', [false, true, undefined]).
combine('stencilFrontDepthFailOp', ['keep', 'zero']).
combine('stencilBackDepthFailOp', ['keep', 'zero'])
).
fn((t) => {
  const {
    isAsync,
    format,
    depthCompare,
    depthWriteEnabled,
    stencilFrontDepthFailOp,
    stencilBackDepthFailOp
  } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  const descriptor = t.getDescriptor({
    depthStencil: {
      format,
      depthCompare,
      depthWriteEnabled,
      stencilFront: { depthFailOp: stencilFrontDepthFailOp },
      stencilBack: { depthFailOp: stencilBackDepthFailOp }
    }
  });

  const depthFailOpsAreKeep =
  stencilFrontDepthFailOp === 'keep' && stencilBackDepthFailOp === 'keep';
  const stencilStateIsDefault = depthFailOpsAreKeep;
  let success = true;
  if (depthWriteEnabled || depthCompare && depthCompare !== 'always') {
    if (!isDepthTextureFormat(format)) success = false;
  }
  if (!stencilStateIsDefault) {
    if (!isStencilTextureFormat(format)) success = false;
  }
  if (isDepthTextureFormat(format)) {
    if (depthWriteEnabled === undefined) success = false;
    if (depthWriteEnabled || !depthFailOpsAreKeep) {
      if (depthCompare === undefined) success = false;
    }
  }

  vtu.doCreateRenderPipelineTest(t, isAsync, success, descriptor);
});

g.test('depthWriteEnabled_optional').
desc(
  `The depthWriteEnabled in depthStencilState is optional for stencil-only formats but required for formats with a depth.`
).
params((u) => u.combine('isAsync', [false, true]).combine('format', kDepthStencilFormats)).
fn((t) => {
  const { isAsync, format } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  const descriptor = t.getDescriptor({
    depthStencil: { format, depthCompare: 'always', depthWriteEnabled: undefined }
  });

  vtu.doCreateRenderPipelineTest(t, isAsync, !isDepthTextureFormat(format), descriptor);
});

g.test('depth_test').
desc(
  `Depth aspect must be contained in the format if depth test is enabled in depthStencilState.`
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', kDepthStencilFormats).
combine('depthCompare', kCompareFunctions)
).
fn((t) => {
  const { isAsync, format, depthCompare } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const descriptor = t.getDescriptor({
    depthStencil: { format, depthCompare, depthWriteEnabled: false }
  });

  const depthTestEnabled = depthCompare !== undefined && depthCompare !== 'always';
  vtu.doCreateRenderPipelineTest(
    t,
    isAsync,
    !depthTestEnabled || isDepthTextureFormat(format),
    descriptor
  );
});

g.test('depth_write').
desc(
  `Depth aspect must be contained in the format if depth write is enabled in depthStencilState.`
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', kDepthStencilFormats).
combine('depthWriteEnabled', [false, true])
).
fn((t) => {
  const { isAsync, format, depthWriteEnabled } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const descriptor = t.getDescriptor({
    depthStencil: { format, depthWriteEnabled, depthCompare: 'always' }
  });
  vtu.doCreateRenderPipelineTest(
    t,
    isAsync,
    !depthWriteEnabled || isDepthTextureFormat(format),
    descriptor
  );
});

g.test('depth_write,frag_depth').
desc(`Depth aspect must be contained in the format if frag_depth is written in fragment stage.`).
params((u) =>
u.combine('isAsync', [false, true]).combine('format', [undefined, ...kDepthStencilFormats])
).
fn((t) => {
  const { isAsync, format } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const descriptor = t.getDescriptor({
    // Keep one color target so that the pipeline is still valid with no depth stencil target.
    targets: [{ format: 'rgba8unorm' }],
    depthStencil: format ?
    { format, depthWriteEnabled: true, depthCompare: 'always' } :
    undefined,
    fragmentShaderCode: getFragmentShaderCodeWithOutput(
      [{ values: [1, 1, 1, 1], plainType: 'f32', componentCount: 4 }],
      { value: 0.5 }
    )
  });

  const hasDepth = format ? isDepthTextureFormat(format) : false;
  vtu.doCreateRenderPipelineTest(t, isAsync, hasDepth, descriptor);
});

g.test('depth_bias').
desc(`Depth bias parameters are only valid with triangle topologies.`).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('topology', kPrimitiveTopology).
beginSubcases().
combineWithParams([
{},
{ depthBias: -1 },
{ depthBias: 0 },
{ depthBias: 1 },
{ depthBiasSlopeScale: -1 },
{ depthBiasSlopeScale: 0 },
{ depthBiasSlopeScale: 1 },
{ depthBiasClamp: -1 },
{ depthBiasClamp: 0 },
{ depthBiasClamp: 1 }]
)
).
fn((t) => {
  const { isAsync, topology, depthBias, depthBiasSlopeScale, depthBiasClamp } = t.params;

  if (t.isCompatibility && !!depthBiasClamp) {
    t.skip('depthBiasClamp must be 0 on compatibility mode');
  }

  const isTriangleTopology = topology === 'triangle-list' || topology === 'triangle-strip';
  const hasDepthBias = !!depthBias || !!depthBiasSlopeScale || !!depthBiasClamp;
  const shouldSucceed = !hasDepthBias || isTriangleTopology;

  const descriptor = t.getDescriptor({
    primitive: { topology },
    depthStencil: {
      format: 'depth24plus',
      depthWriteEnabled: true,
      depthCompare: 'less-equal',
      depthBias,
      depthBiasSlopeScale,
      depthBiasClamp
    }
  });
  vtu.doCreateRenderPipelineTest(t, isAsync, shouldSucceed, descriptor);
});

g.test('stencil_test').
desc(
  `Stencil aspect must be contained in the format if stencil test is enabled in depthStencilState.`
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', kDepthStencilFormats).
combine('face', ['front', 'back']).
combine('compare', [undefined, ...kCompareFunctions])
).
fn((t) => {
  const { isAsync, format, face, compare } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  let descriptor;
  if (face === 'front') {
    descriptor = t.getDescriptor({
      depthStencil: {
        format,
        depthWriteEnabled: false,
        depthCompare: 'always',
        stencilFront: { compare }
      }
    });
  } else {
    descriptor = t.getDescriptor({
      depthStencil: {
        format,
        depthWriteEnabled: false,
        depthCompare: 'always',
        stencilBack: { compare }
      }
    });
  }

  const stencilTestEnabled = compare !== undefined && compare !== 'always';
  vtu.doCreateRenderPipelineTest(
    t,
    isAsync,
    !stencilTestEnabled || isStencilTextureFormat(format),
    descriptor
  );
});

g.test('stencil_write').
desc(
  `Stencil aspect must be contained in the format if stencil write is enabled in depthStencilState.`
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', kDepthStencilFormats).
combine('faceAndOpType', [
'frontFailOp',
'frontDepthFailOp',
'frontPassOp',
'backFailOp',
'backDepthFailOp',
'backPassOp']
).
combine('op', [undefined, ...kStencilOperations])
).
fn((t) => {
  const { isAsync, format, faceAndOpType, op } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const common = {
    format,
    depthWriteEnabled: false,
    depthCompare: 'always'
  };
  let depthStencil;
  switch (faceAndOpType) {
    case 'frontFailOp':
      depthStencil = { ...common, stencilFront: { failOp: op } };
      break;
    case 'frontDepthFailOp':
      depthStencil = { ...common, stencilFront: { depthFailOp: op } };
      break;
    case 'frontPassOp':
      depthStencil = { ...common, stencilFront: { passOp: op } };
      break;
    case 'backFailOp':
      depthStencil = { ...common, stencilBack: { failOp: op } };
      break;
    case 'backDepthFailOp':
      depthStencil = { ...common, stencilBack: { depthFailOp: op } };
      break;
    case 'backPassOp':
      depthStencil = { ...common, stencilBack: { passOp: op } };
      break;
    default:
      unreachable();
  }
  const descriptor = t.getDescriptor({ depthStencil });

  const stencilWriteEnabled = op !== undefined && op !== 'keep';
  vtu.doCreateRenderPipelineTest(
    t,
    isAsync,
    !stencilWriteEnabled || isStencilTextureFormat(format),
    descriptor
  );
});