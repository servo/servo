/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { unreachable } from '../../common/util/util.js';
// TESTING_TODO: This should expand to more canvas types (which will enhance a bunch of tests):
// - canvas element not in dom
// - canvas element in dom
// - offscreen canvas from transferControlToOffscreen from canvas not in dom
// - offscreen canvas from transferControlToOffscreen from canvas in dom
// - offscreen canvas from new OffscreenCanvas
export const kAllCanvasTypes = ['onscreen', 'offscreen'];







/** Valid contextId for HTMLCanvasElement/OffscreenCanvas,
 *  spec: https://html.spec.whatwg.org/multipage/canvas.html#dom-canvas-getcontext
 */
export const kValidCanvasContextIds = [
'2d',
'bitmaprenderer',
'webgl',
'webgl2',
'webgpu'];



/** Create HTMLCanvas/OffscreenCanvas. */
export function createCanvas(
test,
canvasType,
width,
height)
{
  if (canvasType === 'onscreen') {
    if (typeof document !== 'undefined') {
      return createOnscreenCanvas(test, width, height);
    } else {
      test.skip('Cannot create HTMLCanvasElement');
    }
  } else if (canvasType === 'offscreen') {
    if (typeof OffscreenCanvas !== 'undefined') {
      return createOffscreenCanvas(test, width, height);
    } else {
      test.skip('Cannot create an OffscreenCanvas');
    }
  } else {
    unreachable();
  }
}

/** Create HTMLCanvasElement. */
export function createOnscreenCanvas(
test,
width,
height)
{
  let canvas;
  if (typeof document !== 'undefined') {
    canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
  } else {
    test.skip('Cannot create HTMLCanvasElement');
  }
  return canvas;
}

/** Create OffscreenCanvas. */
export function createOffscreenCanvas(
test,
width,
height)
{
  if (typeof OffscreenCanvas === 'undefined') {
    test.skip('OffscreenCanvas is not supported');
  }

  return new OffscreenCanvas(width, height);
}