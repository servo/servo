/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { kUnitCaseParamsBuilder } from '../../../common/framework/params_builder.js';
import { Float16Array } from '../../../external/petamoriken/float16/float16.js';
import { kCanvasAlphaModes, kCanvasColorSpaces } from '../../capability_info.js';

import { runRefTest } from './gpu_ref_test.js';

function bgra8UnormFromRgba8Unorm(rgba8Unorm) {
  const bgra8Unorm = rgba8Unorm.slice();
  for (let i = 0; i < bgra8Unorm.length; i += 4) {
    [bgra8Unorm[i], bgra8Unorm[i + 2]] = [bgra8Unorm[i + 2], bgra8Unorm[i]];
  }
  return bgra8Unorm;
}

function rgba16floatFromRgba8unorm(rgba8Unorm) {
  const rgba16Float = new Float16Array(rgba8Unorm.length);
  for (let i = 0; i < rgba8Unorm.length; ++i) {
    rgba16Float[i] = rgba8Unorm[i] / 255;
  }
  return rgba16Float;
}

function render(device, { canvas, format, alphaMode, colorSpace, textureData }) {
  const context = canvas.getContext('webgpu');
  context.configure({
    device,
    format,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT,
    alphaMode,
    colorSpace,
  });

  const texture = context.getCurrentTexture();
  device.queue.writeTexture({ texture }, textureData, {}, { width: 4, height: 1 });
}

export function runColorSpaceTest(format) {
  runRefTest(async t => {
    const kRGBA8UnormData = new Uint8Array([
      0,
      255,
      0,
      255,
      117,
      251,
      7,
      255,
      170,
      35,
      209,
      255,
      80,
      150,
      200,
      255,
    ]);

    const kBGRA8UnormData = bgra8UnormFromRgba8Unorm(kRGBA8UnormData);
    const kRGBA16FloatData = rgba16floatFromRgba8unorm(kRGBA8UnormData);
    const width = kRGBA8UnormData.length / 4;

    const testData = {
      rgba8unorm: kRGBA8UnormData,
      bgra8unorm: kBGRA8UnormData,
      rgba16float: kRGBA16FloatData,
    };
    const textureData = testData[format].buffer;

    async function createCanvas(creation, alphaMode, format, colorSpace) {
      const canvas = document.createElement('canvas');
      canvas.width = width;
      canvas.height = 1;
      document.body.appendChild(canvas);

      switch (creation) {
        case 'canvas':
          render(t.device, { canvas, format, alphaMode, colorSpace, textureData });
          break;

        case 'transferControlToOffscreen': {
          const offscreenCanvas = canvas.transferControlToOffscreen();
          render(t.device, { canvas: offscreenCanvas, format, alphaMode, colorSpace, textureData });
          break;
        }

        case 'transferControlToOffscreenWorker': {
          const offscreenCanvas = canvas.transferControlToOffscreen();
          const source = `
          ${render.toString()}

          onmessage = async (event) => {
            try {
              const adapter = await navigator.gpu.requestAdapter();
              const device = await adapter.requestDevice();
              render(device, event.data);
              postMessage(true);
            } catch (e) {
              postMessage(false);
            }
          };
          `;
          const blob = new Blob([source], { type: 'application/javascript' });
          const url = URL.createObjectURL(blob);
          const worker = new Worker(url);
          let resolve;
          const promise = new Promise(_resolve => (resolve = _resolve));
          worker.onmessage = event => {
            resolve(event.data);
          };
          worker.postMessage(
            { canvas: offscreenCanvas, format, alphaMode, colorSpace, textureData },
            [offscreenCanvas]
          );

          await promise;
          break;
        }
      }
    }

    const u = kUnitCaseParamsBuilder
      .combine('alphaMode', kCanvasAlphaModes)
      .combine('colorSpace', kCanvasColorSpaces)
      .combine('creation', [
        'canvas',
        'transferControlToOffscreen',
        'transferControlToOffscreenWorker',
      ]);

    for (const { alphaMode, colorSpace, creation } of u) {
      await createCanvas(creation, alphaMode, format, colorSpace);
    }
  });
}
