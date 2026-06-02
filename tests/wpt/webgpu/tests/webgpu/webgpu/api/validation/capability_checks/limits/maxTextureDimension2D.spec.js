/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { getGPU } from '../../../../../common/util/navigator_gpu.js';import { kAllCanvasTypes, createCanvas } from '../../../../util/create_elements.js';
import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';

const limit = 'maxTextureDimension2D';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createTexture,at_over').
desc(`Test using at and over ${limit} limit`).
params(kMaximumLimitBaseParams).
fn(async (t) => {
  const { limitTest, testValueName } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ shouldError, testValue, actualLimit }) => {
      for (let dimensionIndex = 0; dimensionIndex < 2; ++dimensionIndex) {
        const size = [1, 1, 1];
        size[dimensionIndex] = testValue;

        await t.testForValidationErrorWithPossibleOutOfMemoryError(
          () => {
            const texture = t.createTextureTracked({
              size,
              format: 'rgba8unorm',
              usage: GPUTextureUsage.TEXTURE_BINDING
            });

            // MAINTENANCE_TODO: Remove this 'if' once the bug in chrome is fixed
            // This 'if' is only here because of a bug in Chrome
            // that generates an error calling destroy on an invalid texture.
            // This doesn't affect the test but the 'if' should be removed
            // once the Chrome bug is fixed.
            if (!shouldError) {
              texture.destroy();
            }
          },
          shouldError,
          `size: ${size}, actualLimit: ${actualLimit}`
        );
      }
    }
  );
});

g.test('configure,at_over').
desc(`Test using at and over ${limit} limit`).
params(kMaximumLimitBaseParams.combine('canvasType', kAllCanvasTypes)).
fn(async (t) => {
  const { limitTest, testValueName, canvasType } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, shouldError, testValue, actualLimit }) => {
      for (let dimensionIndex = 0; dimensionIndex < 2; ++dimensionIndex) {
        const size = [1, 1];
        size[dimensionIndex] = testValue;

        // This should not fail, even if the size is too large but it might fail
        // if we're in a worker and HTMLCanvasElement does not exist.
        const canvas = createCanvas(t, canvasType, size[0], size[1]);
        if (canvas) {
          const context = canvas.getContext('webgpu');
          t.expect(!!context, 'should not fail to create context even if size is too large');

          await t.testForValidationErrorWithPossibleOutOfMemoryError(
            () => {
              context.configure({
                device,
                format: getGPU(t.rec).getPreferredCanvasFormat()
              });
            },
            shouldError,
            `size: ${size}, actualLimit: ${actualLimit}`
          );
        }
      }
    }
  );
});

g.test('getCurrentTexture,at_over').
desc(`Test using at and over ${limit} limit`).
params(kMaximumLimitBaseParams.combine('canvasType', kAllCanvasTypes)).
fn(async (t) => {
  const { limitTest, testValueName, canvasType } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, shouldError, testValue, actualLimit }) => {
      for (let dimensionIndex = 0; dimensionIndex < 2; ++dimensionIndex) {
        const size = [1, 1];
        size[dimensionIndex] = testValue;

        // Start with a small size so configure will succeed.
        // This should not fail, even if the size is too large but it might fail
        // if we're in a worker and HTMLCanvasElement does not exist.
        const canvas = createCanvas(t, canvasType, 1, 1);
        if (canvas) {
          const context = canvas.getContext('webgpu');
          t.expect(!!context, 'should not fail to create context even if size is too large');

          context.configure({
            device,
            format: getGPU(t.rec).getPreferredCanvasFormat()
          });

          if (canvas) {
            await t.testForValidationErrorWithPossibleOutOfMemoryError(
              () => {
                canvas.width = size[0];
                canvas.height = size[1];
                const texture = context.getCurrentTexture();

                // MAINTENANCE_TODO: Remove this 'if' once the bug in chrome is fixed
                // This 'if' is only here because of a bug in Chrome
                // that generates an error calling destroy on an invalid texture.
                // This doesn't affect the test but the 'if' should be removed
                // once the Chrome bug is fixed.
                if (!shouldError) {
                  texture.destroy();
                }
              },
              shouldError,
              `size: ${size}, actualLimit: ${actualLimit}`
            );
          }
        }
      }
    }
  );
});