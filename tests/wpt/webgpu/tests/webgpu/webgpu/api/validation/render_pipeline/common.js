/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { getTextureFormatType } from '../../../format_info.js';import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';import {
  getFragmentShaderCodeWithOutput,
  getPlainTypeInfo,
  kDefaultVertexShaderCode } from
'../../../util/shader.js';



const values = [0, 1, 0, 1];
export function getDescriptorForCreateRenderPipelineValidationTest(
device,
options =







{})
{
  const {
    primitive = {},
    targets = [{ format: 'rgba8unorm' }],
    multisample = {},
    depthStencil,
    fragmentShaderCode = getFragmentShaderCodeWithOutput([
    {
      values,
      plainType: getPlainTypeInfo(
        getTextureFormatType(targets[0] ? targets[0].format : 'rgba8unorm')
      ),
      componentCount: 4
    }]
    ),
    noFragment = false,
    fragmentConstants = {}
  } = options;

  return {
    vertex: {
      module: device.createShaderModule({
        code: kDefaultVertexShaderCode
      }),
      entryPoint: 'main'
    },
    fragment: noFragment ?
    undefined :
    {
      module: device.createShaderModule({
        code: fragmentShaderCode
      }),
      entryPoint: 'main',
      targets,
      constants: fragmentConstants
    },
    layout: device.createPipelineLayout({ bindGroupLayouts: [] }),
    primitive,
    multisample,
    depthStencil
  };
}

export class CreateRenderPipelineValidationTest extends AllFeaturesMaxLimitsGPUTest {
  getDescriptor(
  options =







  {})
  {
    return getDescriptorForCreateRenderPipelineValidationTest(this.device, options);
  }

  getPipelineLayout() {
    return this.device.createPipelineLayout({ bindGroupLayouts: [] });
  }
}