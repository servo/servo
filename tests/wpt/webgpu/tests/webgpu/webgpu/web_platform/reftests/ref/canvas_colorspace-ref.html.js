/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { kUnitCaseParamsBuilder } from '../../../../common/framework/params_builder.js';
import { kCanvasAlphaModes, kCanvasColorSpaces } from '../../../capability_info.js';

const kRGBAData = new Uint8Array([
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

const width = kRGBAData.length / 4;

function createCanvas(colorSpace) {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = 1;
  const context = canvas.getContext('2d', {
    colorSpace,
  });

  const imgData = context.getImageData(0, 0, width, 1);
  imgData.data.set(kRGBAData);
  context.putImageData(imgData, 0, 0);

  document.body.appendChild(canvas);
}

const u = kUnitCaseParamsBuilder
  .combine('alphaMode', kCanvasAlphaModes)
  .combine('colorSpace', kCanvasColorSpaces)
  .combine('creation', [
    'canvas',
    'transferControlToOffscreen',
    'transferControlToOffscreenWorker',
  ]);

// Generate reference canvases for all combinations from the test.
// We only need colorSpace to generate the correct reference.
for (const { colorSpace } of u) {
  createCanvas(colorSpace);
}
