/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { unreachable } from '../../../../../common/util/util.js';import { AllFeaturesMaxLimitsGPUTest } from '../../../../gpu_test.js';











export class ProgrammableStateTest extends AllFeaturesMaxLimitsGPUTest {
  commonBindGroupLayouts = new Map();

  skipIfNeedsStorageBuffersInFragmentStageAndHaveNone(
  type,
  encoderType)
  {
    if (!this.isCompatibility) {
      return;
    }

    const needsStorageBuffersInFragmentStage =
    type === 'storage' && (encoderType === 'render bundle' || encoderType === 'render pass');

    this.skipIf(
      needsStorageBuffersInFragmentStage &&
      !(this.device.limits.maxStorageBuffersInFragmentStage >= 3),
      `maxStorageBuffersInFragmentStage(${this.device.limits.maxStorageBuffersInFragmentStage}) < 3`
    );
  }

  getBindGroupLayout(
  type,
  visibility)
  {
    const id = `${type}:${visibility}`;
    if (!this.commonBindGroupLayouts.has(id)) {
      this.commonBindGroupLayouts.set(
        id,
        this.device.createBindGroupLayout({
          entries: [
          {
            binding: 0,
            visibility,
            buffer: { type }
          }]

        })
      );
    }
    return this.commonBindGroupLayouts.get(id);
  }

  getVisibilityForEncoderType(encoderType) {
    return encoderType === 'compute pass' ? GPUShaderStage.COMPUTE : GPUShaderStage.FRAGMENT;
  }

  getBindGroupLayouts(
  indices,
  type,
  encoderType)
  {
    const bindGroupLayouts = [];
    const inputType = type === 'storage' ? 'read-only-storage' : 'uniform';
    const visibility = this.getVisibilityForEncoderType(encoderType);
    bindGroupLayouts[indices.a] = this.getBindGroupLayout(inputType, visibility);
    bindGroupLayouts[indices.b] = this.getBindGroupLayout(inputType, visibility);
    if (type === 'storage' || encoderType === 'compute pass') {
      bindGroupLayouts[indices.out] = this.getBindGroupLayout('storage', visibility);
    }
    return bindGroupLayouts;
  }

  createBindGroup(
  buffer,
  type,
  encoderType)
  {
    const visibility = this.getVisibilityForEncoderType(encoderType);
    return this.device.createBindGroup({
      layout: this.getBindGroupLayout(type, visibility),
      entries: [{ binding: 0, resource: { buffer } }]
    });
  }

  setBindGroup(
  encoder,
  index,
  factory)
  {
    encoder.setBindGroup(index, factory(index));
  }

  // Create a compute pipeline that performs an operation on data from two bind groups,
  // then writes the result to a third bind group.
  createBindingStatePipeline(
  encoderType,
  groups,
  type,
  algorithm = 'a.value - b.value')
  {
    switch (encoderType) {
      case 'compute pass':{
          const wgsl = `struct Data {
            value : i32
          };

          @group(${groups.a}) @binding(0) var<${type}> a : Data;
          @group(${groups.b}) @binding(0) var<${type}> b : Data;
          @group(${groups.out}) @binding(0) var<storage, read_write> out : Data;

          @compute @workgroup_size(1) fn main() {
            out.value = ${algorithm};
            return;
          }
        `;

          return this.device.createComputePipeline({
            layout: this.device.createPipelineLayout({
              bindGroupLayouts: this.getBindGroupLayouts(groups, type, encoderType)
            }),
            compute: {
              module: this.device.createShaderModule({
                code: wgsl
              }),
              entryPoint: 'main'
            }
          });
        }
      case 'render pass':
      case 'render bundle':{
          const wgslShaders = {
            vertex: `
            @vertex fn vert_main() -> @builtin(position) vec4<f32> {
              return vec4<f32>(0, 0, 0, 1);
            }
          `,

            fragment: `
            struct Data {
              value : i32
            };

            @group(${groups.a}) @binding(0) var<${type}> a : Data;
            @group(${groups.b}) @binding(0) var<${type}> b : Data;
            @group(${groups.out}) @binding(0) var<storage, read_write> out : Data;

            @fragment fn frag_main_storage() -> @location(0) vec4<i32> {
              out.value = ${algorithm};
              return vec4<i32>(1, 0, 0, 1);
            }
            @fragment fn frag_main_uniform() -> @location(0) vec4<i32> {
              return vec4<i32>(${algorithm});
            }
          `
          };

          return this.device.createRenderPipeline({
            layout: this.device.createPipelineLayout({
              bindGroupLayouts: this.getBindGroupLayouts(groups, type, encoderType)
            }),
            vertex: {
              module: this.device.createShaderModule({
                code: wgslShaders.vertex
              }),
              entryPoint: 'vert_main'
            },
            fragment: {
              module: this.device.createShaderModule({
                code: wgslShaders.fragment
              }),
              entryPoint: type === 'uniform' ? 'frag_main_uniform' : 'frag_main_storage',
              targets: [{ format: 'r32sint' }]
            },
            primitive: { topology: 'point-list' }
          });
        }
      default:
        unreachable();
    }
  }

  createEncoderForStateTest(
  type,
  out,
  ...params)



  {
    const encoderType = params[0];
    const renderTarget = this.createTextureTracked({
      size: [1, 1],
      format: 'r32sint',
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
    });

    // Note: This nightmare of gibberish is trying the result of 2 hours of
    // trying to get typescript to accept the code. Originally the code was
    // effectively just
    //
    //  const { encoder, validateFinishAndSubmit } = this.createEncoder(...);
    //  const fn = (b0, b1) => { validateFinishAndSubmit(b1, b1); if (...) { ... copyT2B ... } }
    //  return { encoder: e__, validateFinishAndSubmit: fn };
    //
    // But TS didn't like it. I couldn't figure out why.
    const encoderAndFinish = this.createEncoder(encoderType, {
      attachmentInfo: { colorFormats: ['r32sint'] },
      targets: [renderTarget.createView()]
    });

    const validateFinishAndSubmit = (
    shouldBeValid,
    submitShouldSucceedIfValid) =>
    {
      encoderAndFinish.validateFinishAndSubmit(shouldBeValid, submitShouldSucceedIfValid);

      if (
      type === 'uniform' && (
      encoderType === 'render pass' || encoderType === 'render bundle'))
      {
        const encoder = this.device.createCommandEncoder();
        encoder.copyTextureToBuffer({ texture: renderTarget }, { buffer: out }, [1, 1]);
        this.device.queue.submit([encoder.finish()]);
      }
    };

    return {
      encoder: encoderAndFinish.encoder,
      validateFinishAndSubmit
    };
  }

  setPipeline(pass, pipeline) {
    if (pass instanceof GPUComputePassEncoder) {
      pass.setPipeline(pipeline);
    } else if (pass instanceof GPURenderPassEncoder || pass instanceof GPURenderBundleEncoder) {
      pass.setPipeline(pipeline);
    }
  }

  dispatchOrDraw(pass) {
    if (pass instanceof GPUComputePassEncoder) {
      pass.dispatchWorkgroups(1);
    } else if (pass instanceof GPURenderPassEncoder) {
      pass.draw(1);
    } else if (pass instanceof GPURenderBundleEncoder) {
      pass.draw(1);
    }
  }
}