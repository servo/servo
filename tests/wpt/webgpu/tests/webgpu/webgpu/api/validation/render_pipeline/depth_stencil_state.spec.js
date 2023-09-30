/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
This test dedicatedly tests validation of GPUDepthStencilState of createRenderPipeline.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { unreachable } from '../../../../common/util/util.js';
import { kCompareFunctions, kStencilOperations } from '../../../capability_info.js';
import { kTextureFormats, kTextureFormatInfo, kDepthStencilFormats } from '../../../format_info.js';
import { getFragmentShaderCodeWithOutput } from '../../../util/shader.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

g.test('format')
  .desc(`The texture format in depthStencilState must be a depth/stencil format.`)
  .params(u => u.combine('isAsync', [false, true]).combine('format', kTextureFormats))
  .beforeAllSubcases(t => {
    const { format } = t.params;
    const info = kTextureFormatInfo[format];
    t.skipIfTextureFormatNotSupported(format);
    t.selectDeviceOrSkipTestCase(info.feature);
  })
  .fn(t => {
    const { isAsync, format } = t.params;
    const info = kTextureFormatInfo[format];

    const descriptor = t.getDescriptor({
      depthStencil: { format, depthWriteEnabled: false, depthCompare: 'always' },
    });

    t.doCreateRenderPipelineTest(isAsync, !!info.depth || !!info.stencil, descriptor);
  });

g.test('depth_test')
  .desc(
    `Depth aspect must be contained in the format if depth test is enabled in depthStencilState.`
  )
  .params(u =>
    u
      .combine('isAsync', [false, true])
      .combine('format', kDepthStencilFormats)
      .combine('depthCompare', kCompareFunctions)
  )
  .beforeAllSubcases(t => {
    const { format } = t.params;
    const info = kTextureFormatInfo[format];
    t.selectDeviceOrSkipTestCase(info.feature);
  })
  .fn(t => {
    const { isAsync, format, depthCompare } = t.params;
    const info = kTextureFormatInfo[format];

    const descriptor = t.getDescriptor({
      depthStencil: { format, depthCompare, depthWriteEnabled: false },
    });

    const depthTestEnabled = depthCompare !== undefined && depthCompare !== 'always';
    t.doCreateRenderPipelineTest(isAsync, !depthTestEnabled || !!info.depth, descriptor);
  });

g.test('depth_write')
  .desc(
    `Depth aspect must be contained in the format if depth write is enabled in depthStencilState.`
  )
  .params(u =>
    u
      .combine('isAsync', [false, true])
      .combine('format', kDepthStencilFormats)
      .combine('depthWriteEnabled', [false, true])
  )
  .beforeAllSubcases(t => {
    const { format } = t.params;
    const info = kTextureFormatInfo[format];
    t.selectDeviceOrSkipTestCase(info.feature);
  })
  .fn(t => {
    const { isAsync, format, depthWriteEnabled } = t.params;
    const info = kTextureFormatInfo[format];

    const descriptor = t.getDescriptor({
      depthStencil: { format, depthWriteEnabled, depthCompare: 'always' },
    });
    t.doCreateRenderPipelineTest(isAsync, !depthWriteEnabled || !!info.depth, descriptor);
  });

g.test('depth_write,frag_depth')
  .desc(`Depth aspect must be contained in the format if frag_depth is written in fragment stage.`)
  .params(u =>
    u.combine('isAsync', [false, true]).combine('format', [undefined, ...kDepthStencilFormats])
  )
  .beforeAllSubcases(t => {
    const { format } = t.params;
    if (format !== undefined) {
      const info = kTextureFormatInfo[format];
      t.selectDeviceOrSkipTestCase(info.feature);
    }
  })
  .fn(t => {
    const { isAsync, format } = t.params;

    const descriptor = t.getDescriptor({
      // Keep one color target so that the pipeline is still valid with no depth stencil target.
      targets: [{ format: 'rgba8unorm' }],
      depthStencil: format
        ? { format, depthWriteEnabled: true, depthCompare: 'always' }
        : undefined,
      fragmentShaderCode: getFragmentShaderCodeWithOutput(
        [{ values: [1, 1, 1, 1], plainType: 'f32', componentCount: 4 }],
        { value: 0.5 }
      ),
    });

    const hasDepth = format ? !!kTextureFormatInfo[format].depth : false;
    t.doCreateRenderPipelineTest(isAsync, hasDepth, descriptor);
  });

g.test('stencil_test')
  .desc(
    `Stencil aspect must be contained in the format if stencil test is enabled in depthStencilState.`
  )
  .params(u =>
    u
      .combine('isAsync', [false, true])
      .combine('format', kDepthStencilFormats)
      .combine('face', ['front', 'back'])
      .combine('compare', [undefined, ...kCompareFunctions])
  )
  .beforeAllSubcases(t => {
    const { format } = t.params;
    const info = kTextureFormatInfo[format];
    t.selectDeviceOrSkipTestCase(info.feature);
  })
  .fn(t => {
    const { isAsync, format, face, compare } = t.params;
    const info = kTextureFormatInfo[format];

    let descriptor;
    if (face === 'front') {
      descriptor = t.getDescriptor({
        depthStencil: {
          format,
          depthWriteEnabled: false,
          depthCompare: 'always',
          stencilFront: { compare },
        },
      });
    } else {
      descriptor = t.getDescriptor({
        depthStencil: {
          format,
          depthWriteEnabled: false,
          depthCompare: 'always',
          stencilBack: { compare },
        },
      });
    }

    const stencilTestEnabled = compare !== undefined && compare !== 'always';
    t.doCreateRenderPipelineTest(isAsync, !stencilTestEnabled || !!info.stencil, descriptor);
  });

g.test('stencil_write')
  .desc(
    `Stencil aspect must be contained in the format if stencil write is enabled in depthStencilState.`
  )
  .params(u =>
    u
      .combine('isAsync', [false, true])
      .combine('format', kDepthStencilFormats)
      .combine('faceAndOpType', [
        'frontFailOp',
        'frontDepthFailOp',
        'frontPassOp',
        'backFailOp',
        'backDepthFailOp',
        'backPassOp',
      ])
      .combine('op', [undefined, ...kStencilOperations])
  )
  .beforeAllSubcases(t => {
    const { format } = t.params;
    const info = kTextureFormatInfo[format];
    t.selectDeviceOrSkipTestCase(info.feature);
  })
  .fn(t => {
    const { isAsync, format, faceAndOpType, op } = t.params;
    const info = kTextureFormatInfo[format];

    const common = {
      format,
      depthWriteEnabled: false,
      depthCompare: 'always',
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
