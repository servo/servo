/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { poptions } from '../../framework/index.js';
// prettier-ignore
export const textureFormatInfo = {
  // Try to keep these manually-formatted in a readable grid.
  // (Note: this list should always match the one in the spec.)
  // 8-bit formats
  'r8unorm': {
    renderable: true,
    color: true
  },
  'r8snorm': {
    renderable: false,
    color: true
  },
  'r8uint': {
    renderable: true,
    color: true
  },
  'r8sint': {
    renderable: true,
    color: true
  },
  // 16-bit formats
  'r16uint': {
    renderable: true,
    color: true
  },
  'r16sint': {
    renderable: true,
    color: true
  },
  'r16float': {
    renderable: true,
    color: true
  },
  'rg8unorm': {
    renderable: true,
    color: true
  },
  'rg8snorm': {
    renderable: false,
    color: true
  },
  'rg8uint': {
    renderable: true,
    color: true
  },
  'rg8sint': {
    renderable: true,
    color: true
  },
  // 32-bit formats
  'r32uint': {
    renderable: true,
    color: true
  },
  'r32sint': {
    renderable: true,
    color: true
  },
  'r32float': {
    renderable: true,
    color: true
  },
  'rg16uint': {
    renderable: true,
    color: true
  },
  'rg16sint': {
    renderable: true,
    color: true
  },
  'rg16float': {
    renderable: true,
    color: true
  },
  'rgba8unorm': {
    renderable: true,
    color: true
  },
  'rgba8unorm-srgb': {
    renderable: true,
    color: true
  },
  'rgba8snorm': {
    renderable: false,
    color: true
  },
  'rgba8uint': {
    renderable: true,
    color: true
  },
  'rgba8sint': {
    renderable: true,
    color: true
  },
  'bgra8unorm': {
    renderable: true,
    color: true
  },
  'bgra8unorm-srgb': {
    renderable: true,
    color: true
  },
  // Packed 32-bit formats
  'rgb10a2unorm': {
    renderable: true,
    color: true
  },
  'rg11b10float': {
    renderable: false,
    color: true
  },
  // 64-bit formats
  'rg32uint': {
    renderable: true,
    color: true
  },
  'rg32sint': {
    renderable: true,
    color: true
  },
  'rg32float': {
    renderable: true,
    color: true
  },
  'rgba16uint': {
    renderable: true,
    color: true
  },
  'rgba16sint': {
    renderable: true,
    color: true
  },
  'rgba16float': {
    renderable: true,
    color: true
  },
  // 128-bit formats
  'rgba32uint': {
    renderable: true,
    color: true
  },
  'rgba32sint': {
    renderable: true,
    color: true
  },
  'rgba32float': {
    renderable: true,
    color: true
  },
  // Depth/stencil formats
  'depth32float': {
    renderable: true,
    color: false
  },
  'depth24plus': {
    renderable: true,
    color: false
  },
  'depth24plus-stencil8': {
    renderable: true,
    color: false
  }
};
export const textureFormats = Object.keys(textureFormatInfo);
export const textureFormatParams = Array.from(poptions('format', textureFormats));
//# sourceMappingURL=format_info.js.map