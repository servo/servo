/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that you can not create a render pipeline with different per target blend state or write mask in compat mode.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { CompatibilityTest } from '../../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);



const cases = {
  default(_targets) {
    return true;
  },
  noBlendTarget0(targets) {
    delete targets[0].blend;
    return false;
  },
  noBlendTarget1(targets) {
    delete targets[2].blend;
    return false;
  },
  colorOperation(targets) {
    targets[2].blend.color.operation = 'subtract';
    return false;
  },
  colorSrcFactor(targets) {
    targets[2].blend.color.srcFactor = 'src-alpha';
    return false;
  },
  colorDstFactor(targets) {
    targets[2].blend.color.dstFactor = 'dst-alpha';
    return false;
  },
  alphaOperation(targets) {
    targets[2].blend.alpha.operation = 'subtract';
    return false;
  },
  alphaSrcFactor(targets) {
    targets[2].blend.alpha.srcFactor = 'src-alpha';
    return false;
  },
  alphaDstFactor(targets) {
    targets[2].blend.alpha.dstFactor = 'dst-alpha';
    return false;
  },
  writeMask(targets) {
    targets[2].writeMask = GPUColorWrite.GREEN;
    return false;
  }
};
const caseNames = keysOf(cases);

g.test('colorState').
desc(
  `
Tests that you can not create a render pipeline with different per target blend state or write mask in compat mode.

- Test no blend state vs some blend state
- Test different operation, srcFactor, dstFactor for color and alpha
- Test different writeMask
    `
).
params((u) => u.combine('caseName', caseNames)).
fn((t) => {
  const { caseName } = t.params;

  const module = t.device.createShaderModule({
    code: `
        @vertex fn vs() -> @builtin(position) vec4f {
            return vec4f(0);
        }

        struct FragmentOut {
            @location(0) fragColor0 : vec4f,
            @location(1) fragColor1 : vec4f,
            @location(2) fragColor2 : vec4f,
        }

        @fragment fn fs() -> FragmentOut {
            var output : FragmentOut;
            output.fragColor0 = vec4f(0);
            output.fragColor1 = vec4f(0);
            output.fragColor2 = vec4f(0);
            return output;
        }
      `
  });

  const targets = [
  {
    format: 'rgba8unorm',
    blend: {
      color: {},
      alpha: {}
    }
  },
  null,
  {
    format: 'rgba8unorm',
    blend: {
      color: {},
      alpha: {}
    }
  }];


  const pipelineDescriptor = {
    layout: 'auto',
    vertex: {
      module,
      entryPoint: 'vs'
    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets
    }
  };
  const isValid = cases[caseName](targets);
  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () => t.device.createRenderPipeline(pipelineDescriptor),
    !isValid
  );
});