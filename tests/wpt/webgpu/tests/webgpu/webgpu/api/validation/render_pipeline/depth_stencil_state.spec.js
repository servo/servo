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
  kTextureFormatInfo,
  kDepthStencilFormats } from
'../../../format_info.js';
import { getFragmentShaderCodeWithOutput } from '../../../util/shader.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

g.test('format').
desc(`The texture format in depthStencilState must be a depth/stencil format.`).
params((u) =>
u //
.combine('isAsync', [false, true]).
combine('format', kAllTextureFormats)
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.skipIfTextureFormatNotSupported(format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format } = t.params;
  const info = kTextureFormatInfo[format];

  const descriptor = t.getDescriptor({
    depthStencil: { format, depthWriteEnabled: false, depthCompare: 'always' }
  });

  t.doCreateRenderPipelineTest(isAsync, !!info.depth || !!info.stencil, descriptor);
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
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.skipIfTextureFormatNotSupported(format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const {
    isAsync,
    format,
    depthCompare,
    depthWriteEnabled,
    stencilFrontDepthFailOp,
    stencilBackDepthFailOp
  } = t.params;
  const info = kTextureFormatInfo[format];
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
    if (!info.depth) success = false;
  }
  if (!stencilStateIsDefault) {
    if (!info.stencil) success = false;
  }
  if (info.depth) {
    if (depthWriteEnabled === undefined) success = false;
    if (depthWriteEnabled || !depthFailOpsAreKeep) {
      if (depthCompare === undefined) success = false;
    }
  }

  t.doCreateRenderPipelineTest(isAsync, success, descriptor);
});

g.test('depthWriteEnabled_optional').
desc(
  `The depthWriteEnabled in depthStencilState is optional for stencil-only formats but required for formats with a depth.`
).
params((u) => u.combine('isAsync', [false, true]).combine('format', kDepthStencilFormats)).
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.skipIfTextureFormatNotSupported(format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format } = t.params;
  const info = kTextureFormatInfo[format];
  const descriptor = t.getDescriptor({
    depthStencil: { format, depthCompare: 'always', depthWriteEnabled: undefined }
  });

  t.doCreateRenderPipelineTest(isAsync, !info.depth, descriptor);
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
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format, depthCompare } = t.params;
  const info = kTextureFormatInfo[format];

  const descriptor = t.getDescriptor({
    depthStencil: { format, depthCompare, depthWriteEnabled: false }
  });

  const depthTestEnabled = depthCompare !== undefined && depthCompare !== 'always';
  t.doCreateRenderPipelineTest(isAsync, !depthTestEnabled || !!info.depth, descriptor);
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
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format, depthWriteEnabled } = t.params;
  const info = kTextureFormatInfo[format];

  const descriptor = t.getDescriptor({
    depthStencil: { format, depthWriteEnabled, depthCompare: 'always' }
  });
  t.doCreateRenderPipelineTest(isAsync, !depthWriteEnabled || !!info.depth, descriptor);
});

g.test('depth_write,frag_depth').
desc(`Depth aspect must be contained in the format if frag_depth is written in fragment stage.`).
params((u) =>
u.combine('isAsync', [false, true]).combine('format', [undefined, ...kDepthStencilFormats])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  if (format !== undefined) {
    const info = kTextureFormatInfo[format];
    t.selectDeviceOrSkipTestCase(info.feature);
  }
}).
fn((t) => {
  const { isAsync, format } = t.params;

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

  const hasDepth = format ? !!kTextureFormatInfo[format].depth : false;
  t.doCreateRenderPipelineTest(isAsync, hasDepth, descriptor);
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
  t.doCreateRenderPipelineTest(isAsync, shouldSucceed, descriptor);
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
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format, face, compare } = t.params;
  const info = kTextureFormatInfo[format];

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
  t.doCreateRenderPipelineTest(isAsync, !stencilTestEnabled || !!info.stencil, descriptor);
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
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format, faceAndOpType, op } = t.params;
  const info = kTextureFormatInfo[format];

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
  t.doCreateRenderPipelineTest(isAsync, !stencilWriteEnabled || !!info.stencil, descriptor);
});