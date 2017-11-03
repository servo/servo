/*
 * Copyright 2010 The Closure Compiler Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// WebGL 2.0 portions:

/*
** Copyright (c) 2015 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/

/**
 * @fileoverview Definitions for WebGL functions as described at
 * http://www.khronos.org/registry/webgl/specs/latest/1.0 and
 * http://www.khronos.org/registry/webgl/specs/latest/2.0
 *
 * This file is current up to the WebGL 2.0 spec, including extensions.
 *
 * This relies on html5.js being included for Canvas and Typed Array support.
 *
 * This includes some extensions defined at
 * http://www.khronos.org/registry/webgl/extensions/
 *
 * This file will be merged back into the Closure workspace as soon as
 * the WebGL 2.0 changes have been fully tested.
 *
 * @externs
 */

/**
 * @constructor
 * @noalias
 * @private
 */
function WebGLRenderingContextBase() {}

/** @typedef {number} */
WebGLRenderingContextBase.GLenum;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_BUFFER_BIT;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BUFFER_BIT;

/** @type {number} */
WebGLRenderingContextBase.COLOR_BUFFER_BIT;

/** @type {number} */
WebGLRenderingContextBase.POINTS;

/** @type {number} */
WebGLRenderingContextBase.LINES;

/** @type {number} */
WebGLRenderingContextBase.LINE_LOOP;

/** @type {number} */
WebGLRenderingContextBase.LINE_STRIP;

/** @type {number} */
WebGLRenderingContextBase.TRIANGLES;

/** @type {number} */
WebGLRenderingContextBase.TRIANGLE_STRIP;

/** @type {number} */
WebGLRenderingContextBase.TRIANGLE_FAN;

/** @type {number} */
WebGLRenderingContextBase.ZERO;

/** @type {number} */
WebGLRenderingContextBase.ONE;

/** @type {number} */
WebGLRenderingContextBase.SRC_COLOR;

/** @type {number} */
WebGLRenderingContextBase.ONE_MINUS_SRC_COLOR;

/** @type {number} */
WebGLRenderingContextBase.SRC_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.ONE_MINUS_SRC_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.DST_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.ONE_MINUS_DST_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.DST_COLOR;

/** @type {number} */
WebGLRenderingContextBase.ONE_MINUS_DST_COLOR;

/** @type {number} */
WebGLRenderingContextBase.SRC_ALPHA_SATURATE;

/** @type {number} */
WebGLRenderingContextBase.FUNC_ADD;

/** @type {number} */
WebGLRenderingContextBase.BLEND_EQUATION;

/** @type {number} */
WebGLRenderingContextBase.BLEND_EQUATION_RGB;

/** @type {number} */
WebGLRenderingContextBase.BLEND_EQUATION_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.FUNC_SUBTRACT;

/** @type {number} */
WebGLRenderingContextBase.FUNC_REVERSE_SUBTRACT;

/** @type {number} */
WebGLRenderingContextBase.BLEND_DST_RGB;

/** @type {number} */
WebGLRenderingContextBase.BLEND_SRC_RGB;

/** @type {number} */
WebGLRenderingContextBase.BLEND_DST_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.BLEND_SRC_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.CONSTANT_COLOR;

/** @type {number} */
WebGLRenderingContextBase.ONE_MINUS_CONSTANT_COLOR;

/** @type {number} */
WebGLRenderingContextBase.CONSTANT_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.ONE_MINUS_CONSTANT_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.BLEND_COLOR;

/** @type {number} */
WebGLRenderingContextBase.ARRAY_BUFFER;

/** @type {number} */
WebGLRenderingContextBase.ELEMENT_ARRAY_BUFFER;

/** @type {number} */
WebGLRenderingContextBase.ARRAY_BUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.ELEMENT_ARRAY_BUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.STREAM_DRAW;

/** @type {number} */
WebGLRenderingContextBase.STATIC_DRAW;

/** @type {number} */
WebGLRenderingContextBase.DYNAMIC_DRAW;

/** @type {number} */
WebGLRenderingContextBase.BUFFER_SIZE;

/** @type {number} */
WebGLRenderingContextBase.BUFFER_USAGE;

/** @type {number} */
WebGLRenderingContextBase.CURRENT_VERTEX_ATTRIB;

/** @type {number} */
WebGLRenderingContextBase.FRONT;

/** @type {number} */
WebGLRenderingContextBase.BACK;

/** @type {number} */
WebGLRenderingContextBase.FRONT_AND_BACK;

/** @type {number} */
WebGLRenderingContextBase.CULL_FACE;

/** @type {number} */
WebGLRenderingContextBase.BLEND;

/** @type {number} */
WebGLRenderingContextBase.DITHER;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_TEST;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_TEST;

/** @type {number} */
WebGLRenderingContextBase.SCISSOR_TEST;

/** @type {number} */
WebGLRenderingContextBase.POLYGON_OFFSET_FILL;

/** @type {number} */
WebGLRenderingContextBase.SAMPLE_ALPHA_TO_COVERAGE;

/** @type {number} */
WebGLRenderingContextBase.SAMPLE_COVERAGE;

/** @type {number} */
WebGLRenderingContextBase.NO_ERROR;

/** @type {number} */
WebGLRenderingContextBase.INVALID_ENUM;

/** @type {number} */
WebGLRenderingContextBase.INVALID_VALUE;

/** @type {number} */
WebGLRenderingContextBase.INVALID_OPERATION;

/** @type {number} */
WebGLRenderingContextBase.OUT_OF_MEMORY;

/** @type {number} */
WebGLRenderingContextBase.CW;

/** @type {number} */
WebGLRenderingContextBase.CCW;

/** @type {number} */
WebGLRenderingContextBase.LINE_WIDTH;

/** @type {number} */
WebGLRenderingContextBase.ALIASED_POINT_SIZE_RANGE;

/** @type {number} */
WebGLRenderingContextBase.ALIASED_LINE_WIDTH_RANGE;

/** @type {number} */
WebGLRenderingContextBase.CULL_FACE_MODE;

/** @type {number} */
WebGLRenderingContextBase.FRONT_FACE;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_RANGE;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_WRITEMASK;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_CLEAR_VALUE;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_FUNC;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_CLEAR_VALUE;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_FUNC;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_FAIL;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_PASS_DEPTH_FAIL;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_PASS_DEPTH_PASS;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_REF;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_VALUE_MASK;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_WRITEMASK;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BACK_FUNC;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BACK_FAIL;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BACK_PASS_DEPTH_FAIL;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BACK_PASS_DEPTH_PASS;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BACK_REF;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BACK_VALUE_MASK;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BACK_WRITEMASK;

/** @type {number} */
WebGLRenderingContextBase.VIEWPORT;

/** @type {number} */
WebGLRenderingContextBase.SCISSOR_BOX;

/** @type {number} */
WebGLRenderingContextBase.COLOR_CLEAR_VALUE;

/** @type {number} */
WebGLRenderingContextBase.COLOR_WRITEMASK;

/** @type {number} */
WebGLRenderingContextBase.UNPACK_ALIGNMENT;

/** @type {number} */
WebGLRenderingContextBase.PACK_ALIGNMENT;

/** @type {number} */
WebGLRenderingContextBase.MAX_TEXTURE_SIZE;

/** @type {number} */
WebGLRenderingContextBase.MAX_VIEWPORT_DIMS;

/** @type {number} */
WebGLRenderingContextBase.SUBPIXEL_BITS;

/** @type {number} */
WebGLRenderingContextBase.RED_BITS;

/** @type {number} */
WebGLRenderingContextBase.GREEN_BITS;

/** @type {number} */
WebGLRenderingContextBase.BLUE_BITS;

/** @type {number} */
WebGLRenderingContextBase.ALPHA_BITS;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_BITS;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_BITS;

/** @type {number} */
WebGLRenderingContextBase.POLYGON_OFFSET_UNITS;

/** @type {number} */
WebGLRenderingContextBase.POLYGON_OFFSET_FACTOR;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_BINDING_2D;

/** @type {number} */
WebGLRenderingContextBase.SAMPLE_BUFFERS;

/** @type {number} */
WebGLRenderingContextBase.SAMPLES;

/** @type {number} */
WebGLRenderingContextBase.SAMPLE_COVERAGE_VALUE;

/** @type {number} */
WebGLRenderingContextBase.SAMPLE_COVERAGE_INVERT;

/** @type {number} */
WebGLRenderingContextBase.COMPRESSED_TEXTURE_FORMATS;

/** @type {number} */
WebGLRenderingContextBase.DONT_CARE;

/** @type {number} */
WebGLRenderingContextBase.FASTEST;

/** @type {number} */
WebGLRenderingContextBase.NICEST;

/** @type {number} */
WebGLRenderingContextBase.GENERATE_MIPMAP_HINT;

/** @type {number} */
WebGLRenderingContextBase.BYTE;

/** @type {number} */
WebGLRenderingContextBase.UNSIGNED_BYTE;

/** @type {number} */
WebGLRenderingContextBase.SHORT;

/** @type {number} */
WebGLRenderingContextBase.UNSIGNED_SHORT;

/** @type {number} */
WebGLRenderingContextBase.INT;

/** @type {number} */
WebGLRenderingContextBase.UNSIGNED_INT;

/** @type {number} */
WebGLRenderingContextBase.FLOAT;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_COMPONENT;

/** @type {number} */
WebGLRenderingContextBase.ALPHA;

/** @type {number} */
WebGLRenderingContextBase.RGB;

/** @type {number} */
WebGLRenderingContextBase.RGBA;

/** @type {number} */
WebGLRenderingContextBase.LUMINANCE;

/** @type {number} */
WebGLRenderingContextBase.LUMINANCE_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.UNSIGNED_SHORT_4_4_4_4;

/** @type {number} */
WebGLRenderingContextBase.UNSIGNED_SHORT_5_5_5_1;

/** @type {number} */
WebGLRenderingContextBase.UNSIGNED_SHORT_5_6_5;

/** @type {number} */
WebGLRenderingContextBase.FRAGMENT_SHADER;

/** @type {number} */
WebGLRenderingContextBase.VERTEX_SHADER;

/** @type {number} */
WebGLRenderingContextBase.MAX_VERTEX_ATTRIBS;

/** @type {number} */
WebGLRenderingContextBase.MAX_VERTEX_UNIFORM_VECTORS;

/** @type {number} */
WebGLRenderingContextBase.MAX_VARYING_VECTORS;

/** @type {number} */
WebGLRenderingContextBase.MAX_COMBINED_TEXTURE_IMAGE_UNITS;

/** @type {number} */
WebGLRenderingContextBase.MAX_VERTEX_TEXTURE_IMAGE_UNITS;

/** @type {number} */
WebGLRenderingContextBase.MAX_TEXTURE_IMAGE_UNITS;

/** @type {number} */
WebGLRenderingContextBase.MAX_FRAGMENT_UNIFORM_VECTORS;

/** @type {number} */
WebGLRenderingContextBase.SHADER_TYPE;

/** @type {number} */
WebGLRenderingContextBase.DELETE_STATUS;

/** @type {number} */
WebGLRenderingContextBase.LINK_STATUS;

/** @type {number} */
WebGLRenderingContextBase.VALIDATE_STATUS;

/** @type {number} */
WebGLRenderingContextBase.ATTACHED_SHADERS;

/** @type {number} */
WebGLRenderingContextBase.ACTIVE_UNIFORMS;

/** @type {number} */
WebGLRenderingContextBase.ACTIVE_ATTRIBUTES;

/** @type {number} */
WebGLRenderingContextBase.SHADING_LANGUAGE_VERSION;

/** @type {number} */
WebGLRenderingContextBase.CURRENT_PROGRAM;

/** @type {number} */
WebGLRenderingContextBase.NEVER;

/** @type {number} */
WebGLRenderingContextBase.LESS;

/** @type {number} */
WebGLRenderingContextBase.EQUAL;

/** @type {number} */
WebGLRenderingContextBase.LEQUAL;

/** @type {number} */
WebGLRenderingContextBase.GREATER;

/** @type {number} */
WebGLRenderingContextBase.NOTEQUAL;

/** @type {number} */
WebGLRenderingContextBase.GEQUAL;

/** @type {number} */
WebGLRenderingContextBase.ALWAYS;

/** @type {number} */
WebGLRenderingContextBase.KEEP;

/** @type {number} */
WebGLRenderingContextBase.REPLACE;

/** @type {number} */
WebGLRenderingContextBase.INCR;

/** @type {number} */
WebGLRenderingContextBase.DECR;

/** @type {number} */
WebGLRenderingContextBase.INVERT;

/** @type {number} */
WebGLRenderingContextBase.INCR_WRAP;

/** @type {number} */
WebGLRenderingContextBase.DECR_WRAP;

/** @type {number} */
WebGLRenderingContextBase.VENDOR;

/** @type {number} */
WebGLRenderingContextBase.RENDERER;

/** @type {number} */
WebGLRenderingContextBase.VERSION;

/** @type {number} */
WebGLRenderingContextBase.NEAREST;

/** @type {number} */
WebGLRenderingContextBase.LINEAR;

/** @type {number} */
WebGLRenderingContextBase.NEAREST_MIPMAP_NEAREST;

/** @type {number} */
WebGLRenderingContextBase.LINEAR_MIPMAP_NEAREST;

/** @type {number} */
WebGLRenderingContextBase.NEAREST_MIPMAP_LINEAR;

/** @type {number} */
WebGLRenderingContextBase.LINEAR_MIPMAP_LINEAR;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_MAG_FILTER;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_MIN_FILTER;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_WRAP_S;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_WRAP_T;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_2D;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_CUBE_MAP;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_BINDING_CUBE_MAP;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_CUBE_MAP_POSITIVE_X;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_CUBE_MAP_NEGATIVE_X;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_CUBE_MAP_POSITIVE_Y;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_CUBE_MAP_NEGATIVE_Y;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_CUBE_MAP_POSITIVE_Z;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE_CUBE_MAP_NEGATIVE_Z;

/** @type {number} */
WebGLRenderingContextBase.MAX_CUBE_MAP_TEXTURE_SIZE;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE0;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE1;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE2;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE3;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE4;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE5;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE6;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE7;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE8;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE9;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE10;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE11;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE12;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE13;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE14;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE15;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE16;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE17;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE18;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE19;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE20;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE21;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE22;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE23;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE24;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE25;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE26;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE27;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE28;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE29;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE30;

/** @type {number} */
WebGLRenderingContextBase.TEXTURE31;

/** @type {number} */
WebGLRenderingContextBase.ACTIVE_TEXTURE;

/** @type {number} */
WebGLRenderingContextBase.REPEAT;

/** @type {number} */
WebGLRenderingContextBase.CLAMP_TO_EDGE;

/** @type {number} */
WebGLRenderingContextBase.MIRRORED_REPEAT;

/** @type {number} */
WebGLRenderingContextBase.FLOAT_VEC2;

/** @type {number} */
WebGLRenderingContextBase.FLOAT_VEC3;

/** @type {number} */
WebGLRenderingContextBase.FLOAT_VEC4;

/** @type {number} */
WebGLRenderingContextBase.INT_VEC2;

/** @type {number} */
WebGLRenderingContextBase.INT_VEC3;

/** @type {number} */
WebGLRenderingContextBase.INT_VEC4;

/** @type {number} */
WebGLRenderingContextBase.BOOL;

/** @type {number} */
WebGLRenderingContextBase.BOOL_VEC2;

/** @type {number} */
WebGLRenderingContextBase.BOOL_VEC3;

/** @type {number} */
WebGLRenderingContextBase.BOOL_VEC4;

/** @type {number} */
WebGLRenderingContextBase.FLOAT_MAT2;

/** @type {number} */
WebGLRenderingContextBase.FLOAT_MAT3;

/** @type {number} */
WebGLRenderingContextBase.FLOAT_MAT4;

/** @type {number} */
WebGLRenderingContextBase.SAMPLER_2D;

/** @type {number} */
WebGLRenderingContextBase.SAMPLER_CUBE;

/** @type {number} */
WebGLRenderingContextBase.VERTEX_ATTRIB_ARRAY_ENABLED;

/** @type {number} */
WebGLRenderingContextBase.VERTEX_ATTRIB_ARRAY_SIZE;

/** @type {number} */
WebGLRenderingContextBase.VERTEX_ATTRIB_ARRAY_STRIDE;

/** @type {number} */
WebGLRenderingContextBase.VERTEX_ATTRIB_ARRAY_TYPE;

/** @type {number} */
WebGLRenderingContextBase.VERTEX_ATTRIB_ARRAY_NORMALIZED;

/** @type {number} */
WebGLRenderingContextBase.VERTEX_ATTRIB_ARRAY_POINTER;

/** @type {number} */
WebGLRenderingContextBase.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.IMPLEMENTATION_COLOR_READ_TYPE;

/** @type {number} */
WebGLRenderingContextBase.IMPLEMENTATION_COLOR_READ_FORMAT;

/** @type {number} */
WebGLRenderingContextBase.COMPILE_STATUS;

/** @type {number} */
WebGLRenderingContextBase.LOW_FLOAT;

/** @type {number} */
WebGLRenderingContextBase.MEDIUM_FLOAT;

/** @type {number} */
WebGLRenderingContextBase.HIGH_FLOAT;

/** @type {number} */
WebGLRenderingContextBase.LOW_INT;

/** @type {number} */
WebGLRenderingContextBase.MEDIUM_INT;

/** @type {number} */
WebGLRenderingContextBase.HIGH_INT;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER;

/** @type {number} */
WebGLRenderingContextBase.RGBA4;

/** @type {number} */
WebGLRenderingContextBase.RGB5_A1;

/** @type {number} */
WebGLRenderingContextBase.RGB565;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_COMPONENT16;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_INDEX;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_INDEX8;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_STENCIL;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_WIDTH;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_HEIGHT;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_INTERNAL_FORMAT;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_RED_SIZE;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_GREEN_SIZE;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_BLUE_SIZE;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_ALPHA_SIZE;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_DEPTH_SIZE;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_STENCIL_SIZE;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE;

/** @type {number} */
WebGLRenderingContextBase.COLOR_ATTACHMENT0;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.STENCIL_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.DEPTH_STENCIL_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.NONE;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_COMPLETE;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_INCOMPLETE_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_INCOMPLETE_DIMENSIONS;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_UNSUPPORTED;

/** @type {number} */
WebGLRenderingContextBase.FRAMEBUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.RENDERBUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.MAX_RENDERBUFFER_SIZE;

/** @type {number} */
WebGLRenderingContextBase.INVALID_FRAMEBUFFER_OPERATION;

/** @type {number} */
WebGLRenderingContextBase.UNPACK_FLIP_Y_WEBGL;

/** @type {number} */
WebGLRenderingContextBase.UNPACK_PREMULTIPLY_ALPHA_WEBGL;

/** @type {number} */
WebGLRenderingContextBase.CONTEXT_LOST_WEBGL;

/** @type {number} */
WebGLRenderingContextBase.UNPACK_COLORSPACE_CONVERSION_WEBGL;

/** @type {number} */
WebGLRenderingContextBase.BROWSER_DEFAULT_WEBGL;


/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_BUFFER_BIT;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BUFFER_BIT;

/** @type {number} */
WebGLRenderingContextBase.prototype.COLOR_BUFFER_BIT;

/** @type {number} */
WebGLRenderingContextBase.prototype.POINTS;

/** @type {number} */
WebGLRenderingContextBase.prototype.LINES;

/** @type {number} */
WebGLRenderingContextBase.prototype.LINE_LOOP;

/** @type {number} */
WebGLRenderingContextBase.prototype.LINE_STRIP;

/** @type {number} */
WebGLRenderingContextBase.prototype.TRIANGLES;

/** @type {number} */
WebGLRenderingContextBase.prototype.TRIANGLE_STRIP;

/** @type {number} */
WebGLRenderingContextBase.prototype.TRIANGLE_FAN;

/** @type {number} */
WebGLRenderingContextBase.prototype.ZERO;

/** @type {number} */
WebGLRenderingContextBase.prototype.ONE;

/** @type {number} */
WebGLRenderingContextBase.prototype.SRC_COLOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.ONE_MINUS_SRC_COLOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.SRC_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.ONE_MINUS_SRC_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.DST_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.ONE_MINUS_DST_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.DST_COLOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.ONE_MINUS_DST_COLOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.SRC_ALPHA_SATURATE;

/** @type {number} */
WebGLRenderingContextBase.prototype.FUNC_ADD;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND_EQUATION;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND_EQUATION_RGB;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND_EQUATION_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.FUNC_SUBTRACT;

/** @type {number} */
WebGLRenderingContextBase.prototype.FUNC_REVERSE_SUBTRACT;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND_DST_RGB;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND_SRC_RGB;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND_DST_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND_SRC_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.CONSTANT_COLOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.ONE_MINUS_CONSTANT_COLOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.CONSTANT_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.ONE_MINUS_CONSTANT_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND_COLOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.ARRAY_BUFFER;

/** @type {number} */
WebGLRenderingContextBase.prototype.ELEMENT_ARRAY_BUFFER;

/** @type {number} */
WebGLRenderingContextBase.prototype.ARRAY_BUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.prototype.ELEMENT_ARRAY_BUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.prototype.STREAM_DRAW;

/** @type {number} */
WebGLRenderingContextBase.prototype.STATIC_DRAW;

/** @type {number} */
WebGLRenderingContextBase.prototype.DYNAMIC_DRAW;

/** @type {number} */
WebGLRenderingContextBase.prototype.BUFFER_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.BUFFER_USAGE;

/** @type {number} */
WebGLRenderingContextBase.prototype.CURRENT_VERTEX_ATTRIB;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRONT;

/** @type {number} */
WebGLRenderingContextBase.prototype.BACK;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRONT_AND_BACK;

/** @type {number} */
WebGLRenderingContextBase.prototype.CULL_FACE;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLEND;

/** @type {number} */
WebGLRenderingContextBase.prototype.DITHER;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_TEST;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_TEST;

/** @type {number} */
WebGLRenderingContextBase.prototype.SCISSOR_TEST;

/** @type {number} */
WebGLRenderingContextBase.prototype.POLYGON_OFFSET_FILL;

/** @type {number} */
WebGLRenderingContextBase.prototype.SAMPLE_ALPHA_TO_COVERAGE;

/** @type {number} */
WebGLRenderingContextBase.prototype.SAMPLE_COVERAGE;

/** @type {number} */
WebGLRenderingContextBase.prototype.NO_ERROR;

/** @type {number} */
WebGLRenderingContextBase.prototype.INVALID_ENUM;

/** @type {number} */
WebGLRenderingContextBase.prototype.INVALID_VALUE;

/** @type {number} */
WebGLRenderingContextBase.prototype.INVALID_OPERATION;

/** @type {number} */
WebGLRenderingContextBase.prototype.OUT_OF_MEMORY;

/** @type {number} */
WebGLRenderingContextBase.prototype.CW;

/** @type {number} */
WebGLRenderingContextBase.prototype.CCW;

/** @type {number} */
WebGLRenderingContextBase.prototype.LINE_WIDTH;

/** @type {number} */
WebGLRenderingContextBase.prototype.ALIASED_POINT_SIZE_RANGE;

/** @type {number} */
WebGLRenderingContextBase.prototype.ALIASED_LINE_WIDTH_RANGE;

/** @type {number} */
WebGLRenderingContextBase.prototype.CULL_FACE_MODE;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRONT_FACE;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_RANGE;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_WRITEMASK;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_CLEAR_VALUE;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_FUNC;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_CLEAR_VALUE;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_FUNC;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_FAIL;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_PASS_DEPTH_FAIL;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_PASS_DEPTH_PASS;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_REF;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_VALUE_MASK;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_WRITEMASK;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BACK_FUNC;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BACK_FAIL;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BACK_PASS_DEPTH_FAIL;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BACK_PASS_DEPTH_PASS;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BACK_REF;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BACK_VALUE_MASK;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BACK_WRITEMASK;

/** @type {number} */
WebGLRenderingContextBase.prototype.VIEWPORT;

/** @type {number} */
WebGLRenderingContextBase.prototype.SCISSOR_BOX;

/** @type {number} */
WebGLRenderingContextBase.prototype.COLOR_CLEAR_VALUE;

/** @type {number} */
WebGLRenderingContextBase.prototype.COLOR_WRITEMASK;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNPACK_ALIGNMENT;

/** @type {number} */
WebGLRenderingContextBase.prototype.PACK_ALIGNMENT;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_TEXTURE_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_VIEWPORT_DIMS;

/** @type {number} */
WebGLRenderingContextBase.prototype.SUBPIXEL_BITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.RED_BITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.GREEN_BITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.BLUE_BITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.ALPHA_BITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_BITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_BITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.POLYGON_OFFSET_UNITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.POLYGON_OFFSET_FACTOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_BINDING_2D;

/** @type {number} */
WebGLRenderingContextBase.prototype.SAMPLE_BUFFERS;

/** @type {number} */
WebGLRenderingContextBase.prototype.SAMPLES;

/** @type {number} */
WebGLRenderingContextBase.prototype.SAMPLE_COVERAGE_VALUE;

/** @type {number} */
WebGLRenderingContextBase.prototype.SAMPLE_COVERAGE_INVERT;

/** @type {number} */
WebGLRenderingContextBase.prototype.COMPRESSED_TEXTURE_FORMATS;

/** @type {number} */
WebGLRenderingContextBase.prototype.DONT_CARE;

/** @type {number} */
WebGLRenderingContextBase.prototype.FASTEST;

/** @type {number} */
WebGLRenderingContextBase.prototype.NICEST;

/** @type {number} */
WebGLRenderingContextBase.prototype.GENERATE_MIPMAP_HINT;

/** @type {number} */
WebGLRenderingContextBase.prototype.BYTE;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNSIGNED_BYTE;

/** @type {number} */
WebGLRenderingContextBase.prototype.SHORT;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNSIGNED_SHORT;

/** @type {number} */
WebGLRenderingContextBase.prototype.INT;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNSIGNED_INT;

/** @type {number} */
WebGLRenderingContextBase.prototype.FLOAT;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_COMPONENT;

/** @type {number} */
WebGLRenderingContextBase.prototype.ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.RGB;

/** @type {number} */
WebGLRenderingContextBase.prototype.RGBA;

/** @type {number} */
WebGLRenderingContextBase.prototype.LUMINANCE;

/** @type {number} */
WebGLRenderingContextBase.prototype.LUMINANCE_ALPHA;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNSIGNED_SHORT_4_4_4_4;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNSIGNED_SHORT_5_5_5_1;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNSIGNED_SHORT_5_6_5;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAGMENT_SHADER;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERTEX_SHADER;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_VERTEX_ATTRIBS;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_VERTEX_UNIFORM_VECTORS;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_VARYING_VECTORS;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_COMBINED_TEXTURE_IMAGE_UNITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_VERTEX_TEXTURE_IMAGE_UNITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_TEXTURE_IMAGE_UNITS;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_FRAGMENT_UNIFORM_VECTORS;

/** @type {number} */
WebGLRenderingContextBase.prototype.SHADER_TYPE;

/** @type {number} */
WebGLRenderingContextBase.prototype.DELETE_STATUS;

/** @type {number} */
WebGLRenderingContextBase.prototype.LINK_STATUS;

/** @type {number} */
WebGLRenderingContextBase.prototype.VALIDATE_STATUS;

/** @type {number} */
WebGLRenderingContextBase.prototype.ATTACHED_SHADERS;

/** @type {number} */
WebGLRenderingContextBase.prototype.ACTIVE_UNIFORMS;

/** @type {number} */
WebGLRenderingContextBase.prototype.ACTIVE_ATTRIBUTES;

/** @type {number} */
WebGLRenderingContextBase.prototype.SHADING_LANGUAGE_VERSION;

/** @type {number} */
WebGLRenderingContextBase.prototype.CURRENT_PROGRAM;

/** @type {number} */
WebGLRenderingContextBase.prototype.NEVER;

/** @type {number} */
WebGLRenderingContextBase.prototype.LESS;

/** @type {number} */
WebGLRenderingContextBase.prototype.EQUAL;

/** @type {number} */
WebGLRenderingContextBase.prototype.LEQUAL;

/** @type {number} */
WebGLRenderingContextBase.prototype.GREATER;

/** @type {number} */
WebGLRenderingContextBase.prototype.NOTEQUAL;

/** @type {number} */
WebGLRenderingContextBase.prototype.GEQUAL;

/** @type {number} */
WebGLRenderingContextBase.prototype.ALWAYS;

/** @type {number} */
WebGLRenderingContextBase.prototype.KEEP;

/** @type {number} */
WebGLRenderingContextBase.prototype.REPLACE;

/** @type {number} */
WebGLRenderingContextBase.prototype.INCR;

/** @type {number} */
WebGLRenderingContextBase.prototype.DECR;

/** @type {number} */
WebGLRenderingContextBase.prototype.INVERT;

/** @type {number} */
WebGLRenderingContextBase.prototype.INCR_WRAP;

/** @type {number} */
WebGLRenderingContextBase.prototype.DECR_WRAP;

/** @type {number} */
WebGLRenderingContextBase.prototype.VENDOR;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERER;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERSION;

/** @type {number} */
WebGLRenderingContextBase.prototype.NEAREST;

/** @type {number} */
WebGLRenderingContextBase.prototype.LINEAR;

/** @type {number} */
WebGLRenderingContextBase.prototype.NEAREST_MIPMAP_NEAREST;

/** @type {number} */
WebGLRenderingContextBase.prototype.LINEAR_MIPMAP_NEAREST;

/** @type {number} */
WebGLRenderingContextBase.prototype.NEAREST_MIPMAP_LINEAR;

/** @type {number} */
WebGLRenderingContextBase.prototype.LINEAR_MIPMAP_LINEAR;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_MAG_FILTER;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_MIN_FILTER;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_WRAP_S;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_WRAP_T;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_2D;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_CUBE_MAP;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_BINDING_CUBE_MAP;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_CUBE_MAP_POSITIVE_X;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_CUBE_MAP_NEGATIVE_X;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_CUBE_MAP_POSITIVE_Y;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_CUBE_MAP_NEGATIVE_Y;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_CUBE_MAP_POSITIVE_Z;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE_CUBE_MAP_NEGATIVE_Z;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_CUBE_MAP_TEXTURE_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE0;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE1;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE2;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE3;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE4;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE5;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE6;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE7;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE8;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE9;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE10;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE11;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE12;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE13;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE14;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE15;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE16;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE17;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE18;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE19;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE20;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE21;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE22;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE23;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE24;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE25;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE26;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE27;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE28;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE29;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE30;

/** @type {number} */
WebGLRenderingContextBase.prototype.TEXTURE31;

/** @type {number} */
WebGLRenderingContextBase.prototype.ACTIVE_TEXTURE;

/** @type {number} */
WebGLRenderingContextBase.prototype.REPEAT;

/** @type {number} */
WebGLRenderingContextBase.prototype.CLAMP_TO_EDGE;

/** @type {number} */
WebGLRenderingContextBase.prototype.MIRRORED_REPEAT;

/** @type {number} */
WebGLRenderingContextBase.prototype.FLOAT_VEC2;

/** @type {number} */
WebGLRenderingContextBase.prototype.FLOAT_VEC3;

/** @type {number} */
WebGLRenderingContextBase.prototype.FLOAT_VEC4;

/** @type {number} */
WebGLRenderingContextBase.prototype.INT_VEC2;

/** @type {number} */
WebGLRenderingContextBase.prototype.INT_VEC3;

/** @type {number} */
WebGLRenderingContextBase.prototype.INT_VEC4;

/** @type {number} */
WebGLRenderingContextBase.prototype.BOOL;

/** @type {number} */
WebGLRenderingContextBase.prototype.BOOL_VEC2;

/** @type {number} */
WebGLRenderingContextBase.prototype.BOOL_VEC3;

/** @type {number} */
WebGLRenderingContextBase.prototype.BOOL_VEC4;

/** @type {number} */
WebGLRenderingContextBase.prototype.FLOAT_MAT2;

/** @type {number} */
WebGLRenderingContextBase.prototype.FLOAT_MAT3;

/** @type {number} */
WebGLRenderingContextBase.prototype.FLOAT_MAT4;

/** @type {number} */
WebGLRenderingContextBase.prototype.SAMPLER_2D;

/** @type {number} */
WebGLRenderingContextBase.prototype.SAMPLER_CUBE;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_ENABLED;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_STRIDE;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_TYPE;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_NORMALIZED;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_POINTER;

/** @type {number} */
WebGLRenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.prototype.IMPLEMENTATION_COLOR_READ_TYPE;

/** @type {number} */
WebGLRenderingContextBase.prototype.IMPLEMENTATION_COLOR_READ_FORMAT;

/** @type {number} */
WebGLRenderingContextBase.prototype.COMPILE_STATUS;

/** @type {number} */
WebGLRenderingContextBase.prototype.LOW_FLOAT;

/** @type {number} */
WebGLRenderingContextBase.prototype.MEDIUM_FLOAT;

/** @type {number} */
WebGLRenderingContextBase.prototype.HIGH_FLOAT;

/** @type {number} */
WebGLRenderingContextBase.prototype.LOW_INT;

/** @type {number} */
WebGLRenderingContextBase.prototype.MEDIUM_INT;

/** @type {number} */
WebGLRenderingContextBase.prototype.HIGH_INT;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER;

/** @type {number} */
WebGLRenderingContextBase.prototype.RGBA4;

/** @type {number} */
WebGLRenderingContextBase.prototype.RGB5_A1;

/** @type {number} */
WebGLRenderingContextBase.prototype.RGB565;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_COMPONENT16;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_INDEX;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_INDEX8;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_STENCIL;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_WIDTH;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_HEIGHT;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_INTERNAL_FORMAT;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_RED_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_GREEN_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_BLUE_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_ALPHA_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_DEPTH_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_STENCIL_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE;

/** @type {number} */
WebGLRenderingContextBase.prototype.COLOR_ATTACHMENT0;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.prototype.STENCIL_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.prototype.DEPTH_STENCIL_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.prototype.NONE;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_COMPLETE;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_INCOMPLETE_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_INCOMPLETE_DIMENSIONS;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_UNSUPPORTED;

/** @type {number} */
WebGLRenderingContextBase.prototype.FRAMEBUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.prototype.RENDERBUFFER_BINDING;

/** @type {number} */
WebGLRenderingContextBase.prototype.MAX_RENDERBUFFER_SIZE;

/** @type {number} */
WebGLRenderingContextBase.prototype.INVALID_FRAMEBUFFER_OPERATION;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNPACK_FLIP_Y_WEBGL;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNPACK_PREMULTIPLY_ALPHA_WEBGL;

/** @type {number} */
WebGLRenderingContextBase.prototype.CONTEXT_LOST_WEBGL;

/** @type {number} */
WebGLRenderingContextBase.prototype.UNPACK_COLORSPACE_CONVERSION_WEBGL;

/** @type {number} */
WebGLRenderingContextBase.prototype.BROWSER_DEFAULT_WEBGL;


/**
 * @type {!HTMLCanvasElement}
 */
WebGLRenderingContextBase.prototype.canvas;

/**
 * @type {number}
 */
WebGLRenderingContextBase.prototype.drawingBufferWidth;

/**
 * @type {number}
 */
WebGLRenderingContextBase.prototype.drawingBufferHeight;

/**
 * @return {!WebGLContextAttributes}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getContextAttributes = function() {};

/**
 * @return {boolean}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.isContextLost = function() {};

/**
 * @return {!Array.<string>}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getSupportedExtensions = function() {};

/**
 * Note that this has side effects by enabling the extension even if the
 * result is not used.
 * @param {string} name
 * @return {Object}
 */
WebGLRenderingContextBase.prototype.getExtension = function(name) {};

/**
 * @param {number} texture
 */
WebGLRenderingContextBase.prototype.activeTexture = function(texture) {};

/**
 * @param {WebGLProgram} program
 * @param {WebGLShader} shader
 */
WebGLRenderingContextBase.prototype.attachShader = function(program, shader) {};

/**
 * @param {WebGLProgram} program
 * @param {number} index
 * @param {string} name
 */
WebGLRenderingContextBase.prototype.bindAttribLocation = function(
    program, index, name) {};

/**
 * @param {number} target
 * @param {WebGLBuffer} buffer
 */
WebGLRenderingContextBase.prototype.bindBuffer = function(target, buffer) {};

/**
 * @param {number} target
 * @param {WebGLFramebuffer} buffer
 */
WebGLRenderingContextBase.prototype.bindFramebuffer = function(target, buffer) {};

/**
 * @param {number} target
 * @param {WebGLRenderbuffer} buffer
 */
WebGLRenderingContextBase.prototype.bindRenderbuffer = function(target, buffer) {};

/**
 * @param {number} target
 * @param {WebGLTexture} texture
 */
WebGLRenderingContextBase.prototype.bindTexture = function(target, texture) {};

/**
 * @param {number} red
 * @param {number} green
 * @param {number} blue
 * @param {number} alpha
 */
WebGLRenderingContextBase.prototype.blendColor = function(
    red, blue, green, alpha) {};

/**
 * @param {number} mode
 */
WebGLRenderingContextBase.prototype.blendEquation = function(mode) {};

/**
 * @param {number} modeRGB
 * @param {number} modeAlpha
 */
WebGLRenderingContextBase.prototype.blendEquationSeparate = function(
    modeRGB, modeAlpha) {};

/**
 * @param {number} sfactor
 * @param {number} dfactor
 */
WebGLRenderingContextBase.prototype.blendFunc = function(sfactor, dfactor) {};

/**
 * @param {number} srcRGB
 * @param {number} dstRGB
 * @param {number} srcAlpha
 * @param {number} dstAlpha
 */
WebGLRenderingContextBase.prototype.blendFuncSeparate = function(
    srcRGB, dstRGB, srcAlpha, dstAlpha) {};

/**
 * @param {number} target
 * @param {ArrayBufferView|ArrayBuffer|number} data
 * @param {number} usage
 */
WebGLRenderingContextBase.prototype.bufferData = function(target, data, usage) {};

/**
 * @param {number} target
 * @param {number} offset
 * @param {ArrayBufferView|ArrayBuffer} data
 */
WebGLRenderingContextBase.prototype.bufferSubData = function(
    target, offset, data) {};

/**
 * @param {number} target
 * @return {number}
 */
WebGLRenderingContextBase.prototype.checkFramebufferStatus = function(target) {};

/**
 * @param {number} mask
 */
WebGLRenderingContextBase.prototype.clear = function(mask) {};

/**
 * @param {number} red
 * @param {number} green
 * @param {number} blue
 * @param {number} alpha
 */
WebGLRenderingContextBase.prototype.clearColor = function(
    red, green, blue, alpha) {};

/**
 * @param {number} depth
 */
WebGLRenderingContextBase.prototype.clearDepth = function(depth) {};

/**
 * @param {number} s
 */
WebGLRenderingContextBase.prototype.clearStencil = function(s) {};

/**
 * @param {boolean} red
 * @param {boolean} green
 * @param {boolean} blue
 * @param {boolean} alpha
 */
WebGLRenderingContextBase.prototype.colorMask = function(
    red, green, blue, alpha) {};

/**
 * @param {WebGLShader} shader
 */
WebGLRenderingContextBase.prototype.compileShader = function(shader) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} internalformat
 * @param {number} width
 * @param {number} height
 * @param {number} border
 * @param {ArrayBufferView} data
 */
WebGLRenderingContextBase.prototype.compressedTexImage2D = function(
    target, level, internalformat, width, height, border, data) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} xoffset
 * @param {number} yoffset
 * @param {number} width
 * @param {number} height
 * @param {number} format
 * @param {ArrayBufferView} data
 */
WebGLRenderingContextBase.prototype.compressedTexSubImage2D = function(
    target, level, xoffset, yoffset, width, height, format, data) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} format
 * @param {number} x
 * @param {number} y
 * @param {number} width
 * @param {number} height
 * @param {number} border
 */
WebGLRenderingContextBase.prototype.copyTexImage2D = function(
    target, level, format, x, y, width, height, border) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} xoffset
 * @param {number} yoffset
 * @param {number} x
 * @param {number} y
 * @param {number} width
 * @param {number} height
 */
WebGLRenderingContextBase.prototype.copyTexSubImage2D = function(
    target, level, xoffset, yoffset, x, y, width, height) {};

/**
 * @return {!WebGLBuffer}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.createBuffer = function() {};

/**
 * @return {!WebGLFramebuffer}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.createFramebuffer = function() {};

/**
 * @return {!WebGLProgram}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.createProgram = function() {};

/**
 * @return {!WebGLRenderbuffer}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.createRenderbuffer = function() {};

/**
 * @param {number} type
 * @return {!WebGLShader}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.createShader = function(type) {};

/**
 * @return {!WebGLTexture}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.createTexture = function() {};

/**
 * @param {number} mode
 */
WebGLRenderingContextBase.prototype.cullFace = function(mode) {};

/**
 * @param {WebGLBuffer} buffer
 */
WebGLRenderingContextBase.prototype.deleteBuffer = function(buffer) {};

/**
 * @param {WebGLFramebuffer} buffer
 */
WebGLRenderingContextBase.prototype.deleteFramebuffer = function(buffer) {};

/**
 * @param {WebGLProgram} program
 */
WebGLRenderingContextBase.prototype.deleteProgram = function(program) {};

/**
 * @param {WebGLRenderbuffer} buffer
 */
WebGLRenderingContextBase.prototype.deleteRenderbuffer = function(buffer) {};

/**
 * @param {WebGLShader} shader
 */
WebGLRenderingContextBase.prototype.deleteShader = function(shader) {};

/**
 * @param {WebGLTexture} texture
 */
WebGLRenderingContextBase.prototype.deleteTexture = function(texture) {};

/**
 * @param {number} func
 */
WebGLRenderingContextBase.prototype.depthFunc = function(func) {};

/**
 * @param {boolean} flag
 */
WebGLRenderingContextBase.prototype.depthMask = function(flag) {};

/**
 * @param {number} nearVal
 * @param {number} farVal
 */
WebGLRenderingContextBase.prototype.depthRange = function(nearVal, farVal) {};

/**
 * @param {WebGLProgram} program
 * @param {WebGLShader} shader
 */
WebGLRenderingContextBase.prototype.detachShader = function(program, shader) {};

/**
 * @param {number} flags
 */
WebGLRenderingContextBase.prototype.disable = function(flags) {};

/**
 * @param {number} index
 */
WebGLRenderingContextBase.prototype.disableVertexAttribArray = function(
    index) {};

/**
 * @param {number} mode
 * @param {number} first
 * @param {number} count
 */
WebGLRenderingContextBase.prototype.drawArrays = function(mode, first, count) {};

/**
 * @param {number} mode
 * @param {number} count
 * @param {number} type
 * @param {number} offset
 */
WebGLRenderingContextBase.prototype.drawElements = function(
    mode, count, type, offset) {};

/**
 * @param {number} cap
 */
WebGLRenderingContextBase.prototype.enable = function(cap) {};

/**
 * @param {number} index
 */
WebGLRenderingContextBase.prototype.enableVertexAttribArray = function(
    index) {};

WebGLRenderingContextBase.prototype.finish = function() {};

WebGLRenderingContextBase.prototype.flush = function() {};

/**
 * @param {number} target
 * @param {number} attachment
 * @param {number} renderbuffertarget
 * @param {WebGLRenderbuffer} renderbuffer
 */
WebGLRenderingContextBase.prototype.framebufferRenderbuffer = function(
    target, attachment, renderbuffertarget, renderbuffer) {};

/**
 * @param {number} target
 * @param {number} attachment
 * @param {number} textarget
 * @param {WebGLTexture} texture
 * @param {number} level
 */
WebGLRenderingContextBase.prototype.framebufferTexture2D = function(
    target, attachment, textarget, texture, level) {};

/**
 * @param {number} mode
 */
WebGLRenderingContextBase.prototype.frontFace = function(mode) {};

/**
 * @param {number} target
 */
WebGLRenderingContextBase.prototype.generateMipmap = function(target) {};

/**
 * @param {WebGLProgram} program
 * @param {number} index
 * @return {WebGLActiveInfo}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getActiveAttrib = function(program, index) {};

/**
 * @param {WebGLProgram} program
 * @param {number} index
 * @return {WebGLActiveInfo}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getActiveUniform = function(program, index) {};

/**
 * @param {WebGLProgram} program
 * @return {!Array.<WebGLShader>}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getAttachedShaders = function(program) {};

/**
 * @param {WebGLProgram} program
 * @param {string} name
 * @return {number}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getAttribLocation = function(program, name) {};

/**
 * @param {number} target
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getBufferParameter = function(target, pname) {};

/**
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getParameter = function(pname) {};

/**
 * @return {number}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getError = function() {};

/**
 * @param {number} target
 * @param {number} attachment
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getFramebufferAttachmentParameter = function(
    target, attachment, pname) {};

/**
 * @param {WebGLProgram} program
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getProgramParameter = function(
    program, pname) {};

/**
 * @param {WebGLProgram} program
 * @return {string}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getProgramInfoLog = function(program) {};

/**
 * @param {number} target
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getRenderbufferParameter = function(
    target, pname) {};

/**
 * @param {WebGLShader} shader
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getShaderParameter = function(shader, pname) {};

/**
 * @param {number} shadertype
 * @param {number} precisiontype
 * @return {WebGLShaderPrecisionFormat}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getShaderPrecisionFormat = function(shadertype,
    precisiontype) {};

/**
 * @param {WebGLShader} shader
 * @return {string}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getShaderInfoLog = function(shader) {};

/**
 * @param {WebGLShader} shader
 * @return {string}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getShaderSource = function(shader) {};

/**
 * @param {number} target
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getTexParameter = function(target, pname) {};

/**
 * @param {WebGLProgram} program
 * @param {WebGLUniformLocation} location
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getUniform = function(program, location) {};

/**
 * @param {WebGLProgram} program
 * @param {string} name
 * @return {WebGLUniformLocation}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getUniformLocation = function(program, name) {};

/**
 * @param {number} index
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getVertexAttrib = function(index, pname) {};

/**
 * @param {number} index
 * @param {number} pname
 * @return {number}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.getVertexAttribOffset = function(
    index, pname) {};

/**
 * @param {number} target
 * @param {number} mode
 */
WebGLRenderingContextBase.prototype.hint = function(target, mode) {};

/**
 * @param {WebGLObject} buffer
 * @return {boolean}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.isBuffer = function(buffer) {};

/**
 * @param {number} cap
 * @return {boolean}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.isEnabled = function(cap) {};

/**
 * @param {WebGLObject} framebuffer
 * @return {boolean}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.isFramebuffer = function(framebuffer) {};

/**
 * @param {WebGLObject} program
 * @return {boolean}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.isProgram = function(program) {};

/**
 * @param {WebGLObject} renderbuffer
 * @return {boolean}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.isRenderbuffer = function(renderbuffer) {};

/**
 * @param {WebGLObject} shader
 * @return {boolean}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.isShader = function(shader) {};

/**
 * @param {WebGLObject} texture
 * @return {boolean}
 * @nosideeffects
 */
WebGLRenderingContextBase.prototype.isTexture = function(texture) {};

/**
 * @param {number} width
 */
WebGLRenderingContextBase.prototype.lineWidth = function(width) {};

/**
 * @param {WebGLProgram} program
 */
WebGLRenderingContextBase.prototype.linkProgram = function(program) {};

/**
 * @param {number} pname
 * @param {number} param
 */
WebGLRenderingContextBase.prototype.pixelStorei = function(pname, param) {};

/**
 * @param {number} factor
 * @param {number} units
 */
WebGLRenderingContextBase.prototype.polygonOffset = function(factor, units) {};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} width
 * @param {number} height
 * @param {number} format
 * @param {number} type
 * @param {ArrayBufferView} pixels
 */
WebGLRenderingContextBase.prototype.readPixels = function(
    x, y, width, height, format, type, pixels) {};

/**
 * @param {number} target
 * @param {number} internalformat
 * @param {number} width
 * @param {number} height
 */
WebGLRenderingContextBase.prototype.renderbufferStorage = function(
    target, internalformat, width, height) {};

/**
 * @param {number} coverage
 * @param {boolean} invert
 */
WebGLRenderingContextBase.prototype.sampleCoverage = function(coverage, invert) {};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} width
 * @param {number} height
 */
WebGLRenderingContextBase.prototype.scissor = function(x, y, width, height) {};

/**
 * @param {WebGLShader} shader
 * @param {string} source
 */
WebGLRenderingContextBase.prototype.shaderSource = function(shader, source) {};

/**
 * @param {number} func
 * @param {number} ref
 * @param {number} mask
 */
WebGLRenderingContextBase.prototype.stencilFunc = function(func, ref, mask) {};

/**
 * @param {number} face
 * @param {number} func
 * @param {number} ref
 * @param {number} mask
 */
WebGLRenderingContextBase.prototype.stencilFuncSeparate = function(
    face, func, ref, mask) {};

/**
 * @param {number} mask
 */
WebGLRenderingContextBase.prototype.stencilMask = function(mask) {};

/**
 * @param {number} face
 * @param {number} mask
 */
WebGLRenderingContextBase.prototype.stencilMaskSeparate = function(face, mask) {};

/**
 * @param {number} fail
 * @param {number} zfail
 * @param {number} zpass
 */
WebGLRenderingContextBase.prototype.stencilOp = function(fail, zfail, zpass) {};

/**
 * @param {number} face
 * @param {number} fail
 * @param {number} zfail
 * @param {number} zpass
 */
WebGLRenderingContextBase.prototype.stencilOpSeparate = function(
    face, fail, zfail, zpass) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} internalformat
 * @param {number} format or width
 * @param {number} type or height
 * @param {ImageData|HTMLImageElement|HTMLCanvasElement|HTMLVideoElement|
 *     number} img or border
 * @param {number=} opt_format
 * @param {number=} opt_type
 * @param {ArrayBufferView=} opt_pixels
 */
WebGLRenderingContextBase.prototype.texImage2D = function(
    target, level, internalformat, format, type, img, opt_format, opt_type,
    opt_pixels) {};

/**
 * @param {number} target
 * @param {number} pname
 * @param {number} param
 */
WebGLRenderingContextBase.prototype.texParameterf = function(
    target, pname, param) {};

/**
 * @param {number} target
 * @param {number} pname
 * @param {number} param
 */
WebGLRenderingContextBase.prototype.texParameteri = function(
    target, pname, param) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} xoffset
 * @param {number} yoffset
 * @param {number} format or width
 * @param {number} type or height
 * @param {ImageData|HTMLImageElement|HTMLCanvasElement|HTMLVideoElement|
 *     number} data or format
 * @param {number=} opt_type
 * @param {ArrayBufferView=} opt_pixels
 */
WebGLRenderingContextBase.prototype.texSubImage2D = function(
    target, level, xoffset, yoffset, format, type, data, opt_type,
    opt_pixels) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} value
 */
WebGLRenderingContextBase.prototype.uniform1f = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Float32Array|Array.<number>} value
 */
WebGLRenderingContextBase.prototype.uniform1fv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} value
 */
WebGLRenderingContextBase.prototype.uniform1i = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Int32Array|Array.<number>} value
 */
WebGLRenderingContextBase.prototype.uniform1iv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} value1
 * @param {number} value2
 */
WebGLRenderingContextBase.prototype.uniform2f = function(
    location, value1, value2) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Float32Array|Array.<number>} value
 */
WebGLRenderingContextBase.prototype.uniform2fv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} value1
 * @param {number} value2
 */
WebGLRenderingContextBase.prototype.uniform2i = function(
    location, value1, value2) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Int32Array|Array.<number>} value
 */
WebGLRenderingContextBase.prototype.uniform2iv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} value1
 * @param {number} value2
 * @param {number} value3
 */
WebGLRenderingContextBase.prototype.uniform3f = function(
    location, value1, value2, value3) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Float32Array|Array.<number>} value
 */
WebGLRenderingContextBase.prototype.uniform3fv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} value1
 * @param {number} value2
 * @param {number} value3
 */
WebGLRenderingContextBase.prototype.uniform3i = function(
    location, value1, value2, value3) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Int32Array|Array.<number>} value
 */
WebGLRenderingContextBase.prototype.uniform3iv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} value1
 * @param {number} value2
 * @param {number} value3
 * @param {number} value4
 */
WebGLRenderingContextBase.prototype.uniform4f = function(
    location, value1, value2, value3, value4) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Float32Array|Array.<number>} value
 */
WebGLRenderingContextBase.prototype.uniform4fv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} value1
 * @param {number} value2
 * @param {number} value3
 * @param {number} value4
 */
WebGLRenderingContextBase.prototype.uniform4i = function(
    location, value1, value2, value3, value4) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Int32Array|Array.<number>} value
 */
WebGLRenderingContextBase.prototype.uniform4iv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} data
 */
WebGLRenderingContextBase.prototype.uniformMatrix2fv = function(
    location, transpose, data) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} data
 */
WebGLRenderingContextBase.prototype.uniformMatrix3fv = function(
    location, transpose, data) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} data
 */
WebGLRenderingContextBase.prototype.uniformMatrix4fv = function(
    location, transpose, data) {};

/**
 * @param {WebGLProgram} program
 */
WebGLRenderingContextBase.prototype.useProgram = function(program) {};

/**
 * @param {WebGLProgram} program
 */
WebGLRenderingContextBase.prototype.validateProgram = function(program) {};

/**
 * @param {number} indx
 * @param {number} x
 */
WebGLRenderingContextBase.prototype.vertexAttrib1f = function(indx, x) {};

/**
 * @param {number} indx
 * @param {Float32Array|Array.<number>} values
 */
WebGLRenderingContextBase.prototype.vertexAttrib1fv = function(indx, values) {};

/**
 * @param {number} indx
 * @param {number} x
 * @param {number} y
 */
WebGLRenderingContextBase.prototype.vertexAttrib2f = function(
    indx, x, y) {};

/**
 * @param {number} indx
 * @param {Float32Array|Array.<number>} values
 */
WebGLRenderingContextBase.prototype.vertexAttrib2fv = function(
    indx, values) {};

/**
 * @param {number} indx
 * @param {number} x
 * @param {number} y
 * @param {number} z
 */
WebGLRenderingContextBase.prototype.vertexAttrib3f = function(
    indx, x, y, z) {};

/**
 * @param {number} indx
 * @param {Float32Array|Array.<number>} values
 */
WebGLRenderingContextBase.prototype.vertexAttrib3fv = function(indx, values) {};

/**
 * @param {number} indx
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @param {number} w
 */
WebGLRenderingContextBase.prototype.vertexAttrib4f = function(
    indx, x, y, z, w) {};

/**
 * @param {number} indx
 * @param {Float32Array|Array.<number>} values
 */
WebGLRenderingContextBase.prototype.vertexAttrib4fv = function(indx, values) {};

/**
 * @param {number} indx
 * @param {number} size
 * @param {number} type
 * @param {boolean} normalized
 * @param {number} stride
 * @param {number} offset
 */
WebGLRenderingContextBase.prototype.vertexAttribPointer = function(
    indx, size, type, normalized, stride, offset) {};

/**
 * @param {number} x
 * @param {number} y
 * @param {number} width
 * @param {number} height
 */
WebGLRenderingContextBase.prototype.viewport = function(x, y, width, height) {};


/**
 * @constructor
 * @noalias
 */
function WebGLContextAttributes() {}

/**
 * @type {boolean}
 */
WebGLContextAttributes.prototype.alpha;

/**
 * @type {boolean}
 */
WebGLContextAttributes.prototype.depth;

/**
 * @type {boolean}
 */
WebGLContextAttributes.prototype.stencil;

/**
 * @type {boolean}
 */
WebGLContextAttributes.prototype.antialias;

/**
 * @type {boolean}
 */
WebGLContextAttributes.prototype.premultipliedAlpha;

/**
 * @type {boolean}
 */
WebGLContextAttributes.prototype.preserveDrawingBuffer;


/**
 * @param {string} eventType
 * @constructor
 * @noalias
 * @extends {Event}
 */
function WebGLContextEvent(eventType) {}

/**
 * @type {string}
 */
WebGLContextEvent.prototype.statusMessage;


/**
 * @constructor
 * @noalias
 */
function WebGLShaderPrecisionFormat() {}

/**
 * @type {number}
 */
WebGLShaderPrecisionFormat.prototype.rangeMin;

/**
 * @type {number}
 */
WebGLShaderPrecisionFormat.prototype.rangeMax;

/**
 * @type {number}
 */
WebGLShaderPrecisionFormat.prototype.precision;


/**
 * @constructor
 * @noalias
 */
function WebGLObject() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLBuffer() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLFramebuffer() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLProgram() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLRenderbuffer() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLShader() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLTexture() {}


/**
 * @constructor
 * @noalias
 */
function WebGLActiveInfo() {}

/** @type {number} */
WebGLActiveInfo.prototype.size;

/** @type {number} */
WebGLActiveInfo.prototype.type;

/** @type {string} */
WebGLActiveInfo.prototype.name;


/**
 * @constructor
 * @noalias
 */
function WebGLUniformLocation() {}


/**
 * @see http://www.khronos.org/registry/webgl/extensions/OES_texture_float/
 * @constructor
 * @noalias
 */
function OES_texture_float() {}


/**
 * @see http://www.khronos.org/registry/webgl/extensions/OES_texture_half_float/
 * @constructor
 * @noalias
 */
function OES_texture_half_float() {}

/** @type {number} */
OES_texture_half_float.prototype.HALF_FLOAT_OES;


/**
 * @see http://www.khronos.org/registry/webgl/extensions/WEBGL_lose_context/
 * @constructor
 * @noalias
 */
function WEBGL_lose_context() {}

WEBGL_lose_context.prototype.loseContext = function() {};

WEBGL_lose_context.prototype.restoreContext = function() {};


/**
 * @see http://www.khronos.org/registry/webgl/extensions/OES_standard_derivatives/
 * @constructor
 * @noalias
 */
function OES_standard_derivatives() {}

/** @type {number} */
OES_standard_derivatives.prototype.FRAGMENT_SHADER_DERIVATIVE_HINT_OES;


/**
 * @see http://www.khronos.org/registry/webgl/extensions/OES_vertex_array_object/
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLVertexArrayObjectOES() {}


/**
 * @see http://www.khronos.org/registry/webgl/extensions/OES_vertex_array_object/
 * @constructor
 * @noalias
 */
function OES_vertex_array_object() {}

/** @type {number} */
OES_vertex_array_object.prototype.VERTEX_ARRAY_BINDING_OES;

/**
 * @return {WebGLVertexArrayObjectOES}
 * @nosideeffects
 */
OES_vertex_array_object.prototype.createVertexArrayOES = function() {};

/**
 * @param {WebGLVertexArrayObjectOES} arrayObject
 */
OES_vertex_array_object.prototype.deleteVertexArrayOES =
    function(arrayObject) {};

/**
 * @param {WebGLVertexArrayObjectOES} arrayObject
 * @return {boolean}
 * @nosideeffects
 */
OES_vertex_array_object.prototype.isVertexArrayOES = function(arrayObject) {};

/**
 * @param {WebGLVertexArrayObjectOES} arrayObject
 */
OES_vertex_array_object.prototype.bindVertexArrayOES = function(arrayObject) {};


/**
 * @see http://www.khronos.org/registry/webgl/extensions/WEBGL_debug_renderer_info/
 * @constructor
 * @noalias
 */
function WEBGL_debug_renderer_info() {}

/** @type {number} */
WEBGL_debug_renderer_info.prototype.UNMASKED_VENDOR_WEBGL;

/** @type {number} */
WEBGL_debug_renderer_info.prototype.UNMASKED_RENDERER_WEBGL;


/**
 * @see http://www.khronos.org/registry/webgl/extensions/WEBGL_debug_shaders/
 * @constructor
 * @noalias
 */
function WEBGL_debug_shaders() {}

/**
 * @param {WebGLShader} shader
 * @return {string}
 * @nosideeffects
 */
WEBGL_debug_shaders.prototype.getTranslatedShaderSource = function(shader) {};


/**
 * @see http://www.khronos.org/registry/webgl/extensions/WEBGL_compressed_texture_s3tc/
 * @constructor
 * @noalias
 */
function WEBGL_compressed_texture_s3tc() {}

/** @type {number} */
WEBGL_compressed_texture_s3tc.prototype.COMPRESSED_RGB_S3TC_DXT1_EXT;

/** @type {number} */
WEBGL_compressed_texture_s3tc.prototype.COMPRESSED_RGBA_S3TC_DXT1_EXT;

/** @type {number} */
WEBGL_compressed_texture_s3tc.prototype.COMPRESSED_RGBA_S3TC_DXT3_EXT;

/** @type {number} */
WEBGL_compressed_texture_s3tc.prototype.COMPRESSED_RGBA_S3TC_DXT5_EXT;


/**
 * @see http://www.khronos.org/registry/webgl/extensions/OES_depth_texture/
 * @constructor
 * @noalias
 */
function OES_depth_texture() {}


/**
 * @see http://www.khronos.org/registry/webgl/extensions/OES_element_index_uint/
 * @constructor
 * @noalias
 */
function OES_element_index_uint() {}


/**
 * @see http://www.khronos.org/registry/webgl/extensions/EXT_texture_filter_anisotropic/
 * @constructor
 * @noalias
 */
function EXT_texture_filter_anisotropic() {}

/** @type {number} */
EXT_texture_filter_anisotropic.prototype.TEXTURE_MAX_ANISOTROPY_EXT;

/** @type {number} */
EXT_texture_filter_anisotropic.prototype.MAX_TEXTURE_MAX_ANISOTROPY_EXT;



/**
 * @see http://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
 * @constructor
 * @noalias
 */
function ANGLE_instanced_arrays() {}


/** @type {number} */
ANGLE_instanced_arrays.prototype.VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE;


/**
 * @param {number} mode Primitive type.
 * @param {number} first First vertex.
 * @param {number} count Number of vertices per instance.
 * @param {number} primcount Number of instances.
 */
ANGLE_instanced_arrays.prototype.drawArraysInstancedANGLE = function(
    mode, first, count, primcount) {};


/**
 * @param {number} mode Primitive type.
 * @param {number} count Number of vertex indices per instance.
 * @param {number} type Type of a vertex index.
 * @param {number} offset Offset to the first vertex index.
 * @param {number} primcount Number of instances.
 */
ANGLE_instanced_arrays.prototype.drawElementsInstancedANGLE = function(
    mode, count, type, offset, primcount) {};


/**
 * @param {number} index Attribute index.
 * @param {number} divisor Instance divisor.
 */
ANGLE_instanced_arrays.prototype.vertexAttribDivisorANGLE = function(
    index, divisor) {};


/**
 * @constructor
 * @noalias
 * @extends {WebGLRenderingContextBase}
 */
function WebGLRenderingContext() {}


/**
 * @constructor
 * @private
 * @noalias
 * @extends {WebGLRenderingContextBase}
 */
function WebGL2RenderingContextBase() {}


/** @type {number} */
WebGL2RenderingContextBase.READ_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.UNPACK_ROW_LENGTH;

/** @type {number} */
WebGL2RenderingContextBase.UNPACK_SKIP_ROWS;

/** @type {number} */
WebGL2RenderingContextBase.UNPACK_SKIP_PIXELS;

/** @type {number} */
WebGL2RenderingContextBase.PACK_ROW_LENGTH;

/** @type {number} */
WebGL2RenderingContextBase.PACK_SKIP_ROWS;

/** @type {number} */
WebGL2RenderingContextBase.PACK_SKIP_PIXELS;

/** @type {number} */
WebGL2RenderingContextBase.COLOR;

/** @type {number} */
WebGL2RenderingContextBase.DEPTH;

/** @type {number} */
WebGL2RenderingContextBase.STENCIL;

/** @type {number} */
WebGL2RenderingContextBase.RED;

/** @type {number} */
WebGL2RenderingContextBase.RGB8;

/** @type {number} */
WebGL2RenderingContextBase.RGBA8;

/** @type {number} */
WebGL2RenderingContextBase.RGB10_A2;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_BINDING_3D;

/** @type {number} */
WebGL2RenderingContextBase.UNPACK_SKIP_IMAGES;

/** @type {number} */
WebGL2RenderingContextBase.UNPACK_IMAGE_HEIGHT;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_3D;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_WRAP_R;

/** @type {number} */
WebGL2RenderingContextBase.MAX_3D_TEXTURE_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_2_10_10_10_REV;

/** @type {number} */
WebGL2RenderingContextBase.MAX_ELEMENTS_VERTICES;

/** @type {number} */
WebGL2RenderingContextBase.MAX_ELEMENTS_INDICES;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_MIN_LOD;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_MAX_LOD;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_BASE_LEVEL;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_MAX_LEVEL;

/** @type {number} */
WebGL2RenderingContextBase.MIN;

/** @type {number} */
WebGL2RenderingContextBase.MAX;

/** @type {number} */
WebGL2RenderingContextBase.DEPTH_COMPONENT24;

/** @type {number} */
WebGL2RenderingContextBase.MAX_TEXTURE_LOD_BIAS;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_COMPARE_MODE;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_COMPARE_FUNC;

/** @type {number} */
WebGL2RenderingContextBase.CURRENT_QUERY;

/** @type {number} */
WebGL2RenderingContextBase.QUERY_RESULT;

/** @type {number} */
WebGL2RenderingContextBase.QUERY_RESULT_AVAILABLE;

/** @type {number} */
WebGL2RenderingContextBase.STREAM_READ;

/** @type {number} */
WebGL2RenderingContextBase.STREAM_COPY;

/** @type {number} */
WebGL2RenderingContextBase.STATIC_READ;

/** @type {number} */
WebGL2RenderingContextBase.STATIC_COPY;

/** @type {number} */
WebGL2RenderingContextBase.DYNAMIC_READ;

/** @type {number} */
WebGL2RenderingContextBase.DYNAMIC_COPY;

/** @type {number} */
WebGL2RenderingContextBase.MAX_DRAW_BUFFERS;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER0;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER1;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER2;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER3;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER4;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER5;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER6;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER7;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER8;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER9;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER10;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER11;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER12;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER13;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER14;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_BUFFER15;

/** @type {number} */
WebGL2RenderingContextBase.MAX_FRAGMENT_UNIFORM_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_VERTEX_UNIFORM_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.SAMPLER_3D;

/** @type {number} */
WebGL2RenderingContextBase.SAMPLER_2D_SHADOW;

/** @type {number} */
WebGL2RenderingContextBase.FRAGMENT_SHADER_DERIVATIVE_HINT;

/** @type {number} */
WebGL2RenderingContextBase.PIXEL_PACK_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.PIXEL_UNPACK_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.PIXEL_PACK_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.PIXEL_UNPACK_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.FLOAT_MAT2x3;

/** @type {number} */
WebGL2RenderingContextBase.FLOAT_MAT2x4;

/** @type {number} */
WebGL2RenderingContextBase.FLOAT_MAT3x2;

/** @type {number} */
WebGL2RenderingContextBase.FLOAT_MAT3x4;

/** @type {number} */
WebGL2RenderingContextBase.FLOAT_MAT4x2;

/** @type {number} */
WebGL2RenderingContextBase.FLOAT_MAT4x3;

/** @type {number} */
WebGL2RenderingContextBase.SRGB;

/** @type {number} */
WebGL2RenderingContextBase.SRGB8;

/** @type {number} */
WebGL2RenderingContextBase.SRGB8_ALPHA8;

/** @type {number} */
WebGL2RenderingContextBase.COMPARE_REF_TO_TEXTURE;

/** @type {number} */
WebGL2RenderingContextBase.RGBA32F;

/** @type {number} */
WebGL2RenderingContextBase.RGB32F;

/** @type {number} */
WebGL2RenderingContextBase.RGBA16F;

/** @type {number} */
WebGL2RenderingContextBase.RGB16F;

/** @type {number} */
WebGL2RenderingContextBase.VERTEX_ATTRIB_ARRAY_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.MAX_ARRAY_TEXTURE_LAYERS;

/** @type {number} */
WebGL2RenderingContextBase.MIN_PROGRAM_TEXEL_OFFSET;

/** @type {number} */
WebGL2RenderingContextBase.MAX_PROGRAM_TEXEL_OFFSET;

/** @type {number} */
WebGL2RenderingContextBase.MAX_VARYING_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_BINDING_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.R11F_G11F_B10F;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_10F_11F_11F_REV;

/** @type {number} */
WebGL2RenderingContextBase.RGB9_E5;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_5_9_9_9_REV;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_BUFFER_MODE;

/** @type {number} */
WebGL2RenderingContextBase.MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_VARYINGS;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_BUFFER_START;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_BUFFER_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN;

/** @type {number} */
WebGL2RenderingContextBase.RASTERIZER_DISCARD;

/** @type {number} */
WebGL2RenderingContextBase.MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS;

/** @type {number} */
WebGL2RenderingContextBase.INTERLEAVED_ATTRIBS;

/** @type {number} */
WebGL2RenderingContextBase.SEPARATE_ATTRIBS;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.RGBA32UI;

/** @type {number} */
WebGL2RenderingContextBase.RGB32UI;

/** @type {number} */
WebGL2RenderingContextBase.RGBA16UI;

/** @type {number} */
WebGL2RenderingContextBase.RGB16UI;

/** @type {number} */
WebGL2RenderingContextBase.RGBA8UI;

/** @type {number} */
WebGL2RenderingContextBase.RGB8UI;

/** @type {number} */
WebGL2RenderingContextBase.RGBA32I;

/** @type {number} */
WebGL2RenderingContextBase.RGB32I;

/** @type {number} */
WebGL2RenderingContextBase.RGBA16I;

/** @type {number} */
WebGL2RenderingContextBase.RGB16I;

/** @type {number} */
WebGL2RenderingContextBase.RGBA8I;

/** @type {number} */
WebGL2RenderingContextBase.RGB8I;

/** @type {number} */
WebGL2RenderingContextBase.RED_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.RGB_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.RGBA_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.SAMPLER_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.SAMPLER_2D_ARRAY_SHADOW;

/** @type {number} */
WebGL2RenderingContextBase.SAMPLER_CUBE_SHADOW;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_VEC2;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_VEC3;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_VEC4;

/** @type {number} */
WebGL2RenderingContextBase.INT_SAMPLER_2D;

/** @type {number} */
WebGL2RenderingContextBase.INT_SAMPLER_3D;

/** @type {number} */
WebGL2RenderingContextBase.INT_SAMPLER_CUBE;

/** @type {number} */
WebGL2RenderingContextBase.INT_SAMPLER_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_SAMPLER_2D;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_SAMPLER_3D;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_SAMPLER_CUBE;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_SAMPLER_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.DEPTH_COMPONENT32F;

/** @type {number} */
WebGL2RenderingContextBase.DEPTH32F_STENCIL8;

/** @type {number} */
WebGL2RenderingContextBase.FLOAT_32_UNSIGNED_INT_24_8_REV;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_RED_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_DEFAULT;

/** @type {number} */
WebGL2RenderingContextBase.DEPTH_STENCIL_ATTACHMENT;

/** @type {number} */
WebGL2RenderingContextBase.DEPTH_STENCIL;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_INT_24_8;

/** @type {number} */
WebGL2RenderingContextBase.DEPTH24_STENCIL8;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNED_NORMALIZED;

// Same as FRAMEBUFFER_BINDING
/** @type {number} */
WebGL2RenderingContextBase.DRAW_FRAMEBUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.READ_FRAMEBUFFER;

/** @type {number} */
WebGL2RenderingContextBase.DRAW_FRAMEBUFFER;

/** @type {number} */
WebGL2RenderingContextBase.READ_FRAMEBUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.RENDERBUFFER_SAMPLES;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER;

/** @type {number} */
WebGL2RenderingContextBase.MAX_COLOR_ATTACHMENTS;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT1;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT2;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT3;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT4;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT5;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT6;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT7;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT8;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT9;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT10;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT11;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT12;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT13;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT14;

/** @type {number} */
WebGL2RenderingContextBase.COLOR_ATTACHMENT15;

/** @type {number} */
WebGL2RenderingContextBase.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE;

/** @type {number} */
WebGL2RenderingContextBase.MAX_SAMPLES;

/** @type {number} */
WebGL2RenderingContextBase.HALF_FLOAT;

/** @type {number} */
WebGL2RenderingContextBase.RG;

/** @type {number} */
WebGL2RenderingContextBase.RG_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.R8;

/** @type {number} */
WebGL2RenderingContextBase.RG8;

/** @type {number} */
WebGL2RenderingContextBase.R16F;

/** @type {number} */
WebGL2RenderingContextBase.R32F;

/** @type {number} */
WebGL2RenderingContextBase.RG16F;

/** @type {number} */
WebGL2RenderingContextBase.RG32F;

/** @type {number} */
WebGL2RenderingContextBase.R8I;

/** @type {number} */
WebGL2RenderingContextBase.R8UI;

/** @type {number} */
WebGL2RenderingContextBase.R16I;

/** @type {number} */
WebGL2RenderingContextBase.R16UI;

/** @type {number} */
WebGL2RenderingContextBase.R32I;

/** @type {number} */
WebGL2RenderingContextBase.R32UI;

/** @type {number} */
WebGL2RenderingContextBase.RG8I;

/** @type {number} */
WebGL2RenderingContextBase.RG8UI;

/** @type {number} */
WebGL2RenderingContextBase.RG16I;

/** @type {number} */
WebGL2RenderingContextBase.RG16UI;

/** @type {number} */
WebGL2RenderingContextBase.RG32I;

/** @type {number} */
WebGL2RenderingContextBase.RG32UI;

/** @type {number} */
WebGL2RenderingContextBase.VERTEX_ARRAY_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.R8_SNORM;

/** @type {number} */
WebGL2RenderingContextBase.RG8_SNORM;

/** @type {number} */
WebGL2RenderingContextBase.RGB8_SNORM;

/** @type {number} */
WebGL2RenderingContextBase.RGBA8_SNORM;

/** @type {number} */
WebGL2RenderingContextBase.SIGNED_NORMALIZED;

/** @type {number} */
WebGL2RenderingContextBase.COPY_READ_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.COPY_WRITE_BUFFER;

// Same as COPY_READ_BUFFER
/** @type {number} */
WebGL2RenderingContextBase.COPY_READ_BUFFER_BINDING;

// Same as COPY_WRITE_BUFFER
/** @type {number} */
WebGL2RenderingContextBase.COPY_WRITE_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BUFFER_START;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BUFFER_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.MAX_VERTEX_UNIFORM_BLOCKS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_FRAGMENT_UNIFORM_BLOCKS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_COMBINED_UNIFORM_BLOCKS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_UNIFORM_BUFFER_BINDINGS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_UNIFORM_BLOCK_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BUFFER_OFFSET_ALIGNMENT;

/** @type {number} */
WebGL2RenderingContextBase.ACTIVE_UNIFORM_BLOCKS;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_TYPE;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BLOCK_INDEX;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_OFFSET;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_ARRAY_STRIDE;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_MATRIX_STRIDE;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_IS_ROW_MAJOR;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BLOCK_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BLOCK_DATA_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BLOCK_ACTIVE_UNIFORMS;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER;

/** @type {number} */
WebGL2RenderingContextBase.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER;

/** @type {number} */
WebGL2RenderingContextBase.INVALID_INDEX;

/** @type {number} */
WebGL2RenderingContextBase.MAX_VERTEX_OUTPUT_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_FRAGMENT_INPUT_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.MAX_SERVER_WAIT_TIMEOUT;

/** @type {number} */
WebGL2RenderingContextBase.OBJECT_TYPE;

/** @type {number} */
WebGL2RenderingContextBase.SYNC_CONDITION;

/** @type {number} */
WebGL2RenderingContextBase.SYNC_STATUS;

/** @type {number} */
WebGL2RenderingContextBase.SYNC_FLAGS;

/** @type {number} */
WebGL2RenderingContextBase.SYNC_FENCE;

/** @type {number} */
WebGL2RenderingContextBase.SYNC_GPU_COMMANDS_COMPLETE;

/** @type {number} */
WebGL2RenderingContextBase.UNSIGNALED;

/** @type {number} */
WebGL2RenderingContextBase.SIGNALED;

/** @type {number} */
WebGL2RenderingContextBase.ALREADY_SIGNALED;

/** @type {number} */
WebGL2RenderingContextBase.TIMEOUT_EXPIRED;

/** @type {number} */
WebGL2RenderingContextBase.CONDITION_SATISFIED;

/** @type {number} */
WebGL2RenderingContextBase.WAIT_FAILED;

/** @type {number} */
WebGL2RenderingContextBase.SYNC_FLUSH_COMMANDS_BIT;

/** @type {number} */
WebGL2RenderingContextBase.VERTEX_ATTRIB_ARRAY_DIVISOR;

/** @type {number} */
WebGL2RenderingContextBase.ANY_SAMPLES_PASSED;

/** @type {number} */
WebGL2RenderingContextBase.ANY_SAMPLES_PASSED_CONSERVATIVE;

/** @type {number} */
WebGL2RenderingContextBase.SAMPLER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.RGB10_A2UI;

/** @type {number} */
WebGL2RenderingContextBase.INT_2_10_10_10_REV;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_PAUSED;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_ACTIVE;

/** @type {number} */
WebGL2RenderingContextBase.TRANSFORM_FEEDBACK_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_IMMUTABLE_FORMAT;

/** @type {number} */
WebGL2RenderingContextBase.MAX_ELEMENT_INDEX;

/** @type {number} */
WebGL2RenderingContextBase.NUM_SAMPLE_COUNTS;

/** @type {number} */
WebGL2RenderingContextBase.TEXTURE_IMMUTABLE_LEVELS;

/** @type {number} */
WebGL2RenderingContextBase.TIMEOUT_IGNORED;


/** @type {number} */
WebGL2RenderingContextBase.prototype.READ_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNPACK_ROW_LENGTH;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNPACK_SKIP_ROWS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNPACK_SKIP_PIXELS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.PACK_ROW_LENGTH;

/** @type {number} */
WebGL2RenderingContextBase.prototype.PACK_SKIP_ROWS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.PACK_SKIP_PIXELS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DEPTH;

/** @type {number} */
WebGL2RenderingContextBase.prototype.STENCIL;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB10_A2;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_BINDING_3D;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNPACK_SKIP_IMAGES;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNPACK_IMAGE_HEIGHT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_3D;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_WRAP_R;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_3D_TEXTURE_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_2_10_10_10_REV;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_ELEMENTS_VERTICES;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_ELEMENTS_INDICES;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_MIN_LOD;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_MAX_LOD;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_BASE_LEVEL;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_MAX_LEVEL;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MIN;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DEPTH_COMPONENT24;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_TEXTURE_LOD_BIAS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_COMPARE_MODE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_COMPARE_FUNC;

/** @type {number} */
WebGL2RenderingContextBase.prototype.CURRENT_QUERY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.QUERY_RESULT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.QUERY_RESULT_AVAILABLE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.STREAM_READ;

/** @type {number} */
WebGL2RenderingContextBase.prototype.STREAM_COPY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.STATIC_READ;

/** @type {number} */
WebGL2RenderingContextBase.prototype.STATIC_COPY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DYNAMIC_READ;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DYNAMIC_COPY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_DRAW_BUFFERS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER0;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER1;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER2;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER3;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER4;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER5;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER6;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER7;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER9;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER10;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER11;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER12;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER13;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER14;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_BUFFER15;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_FRAGMENT_UNIFORM_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_VERTEX_UNIFORM_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SAMPLER_3D;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SAMPLER_2D_SHADOW;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAGMENT_SHADER_DERIVATIVE_HINT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.PIXEL_PACK_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.PIXEL_UNPACK_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.PIXEL_PACK_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.PIXEL_UNPACK_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FLOAT_MAT2x3;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FLOAT_MAT2x4;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FLOAT_MAT3x2;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FLOAT_MAT3x4;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FLOAT_MAT4x2;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FLOAT_MAT4x3;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SRGB;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SRGB8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SRGB8_ALPHA8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COMPARE_REF_TO_TEXTURE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA32F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB32F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA16F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB16F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_ARRAY_TEXTURE_LAYERS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MIN_PROGRAM_TEXEL_OFFSET;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_PROGRAM_TEXEL_OFFSET;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_VARYING_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_BINDING_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R11F_G11F_B10F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_10F_11F_11F_REV;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB9_E5;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_5_9_9_9_REV;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_BUFFER_MODE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_VARYINGS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_BUFFER_START;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_BUFFER_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RASTERIZER_DISCARD;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.INTERLEAVED_ATTRIBS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SEPARATE_ATTRIBS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA32UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB32UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA16UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB16UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA8UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB8UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA32I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB32I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA16I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB16I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA8I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB8I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RED_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SAMPLER_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SAMPLER_2D_ARRAY_SHADOW;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SAMPLER_CUBE_SHADOW;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_VEC2;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_VEC3;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_VEC4;

/** @type {number} */
WebGL2RenderingContextBase.prototype.INT_SAMPLER_2D;

/** @type {number} */
WebGL2RenderingContextBase.prototype.INT_SAMPLER_3D;

/** @type {number} */
WebGL2RenderingContextBase.prototype.INT_SAMPLER_CUBE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.INT_SAMPLER_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_SAMPLER_2D;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_SAMPLER_3D;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_SAMPLER_CUBE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_SAMPLER_2D_ARRAY;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DEPTH_COMPONENT32F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DEPTH32F_STENCIL8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FLOAT_32_UNSIGNED_INT_24_8_REV;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_RED_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_DEFAULT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DEPTH_STENCIL_ATTACHMENT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DEPTH_STENCIL;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_INT_24_8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DEPTH24_STENCIL8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNED_NORMALIZED;

// Same as FRAMEBUFFER_BINDING
/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_FRAMEBUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.READ_FRAMEBUFFER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.DRAW_FRAMEBUFFER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.READ_FRAMEBUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RENDERBUFFER_SAMPLES;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_COLOR_ATTACHMENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT1;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT2;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT3;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT4;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT5;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT6;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT7;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT9;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT10;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT11;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT12;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT13;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT14;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COLOR_ATTACHMENT15;

/** @type {number} */
WebGL2RenderingContextBase.prototype.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_SAMPLES;

/** @type {number} */
WebGL2RenderingContextBase.prototype.HALF_FLOAT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG_INTEGER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG8;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R16F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R32F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG16F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG32F;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R8I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R8UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R16I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R16UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R32I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R32UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG8I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG8UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG16I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG16UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG32I;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG32UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.VERTEX_ARRAY_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.R8_SNORM;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RG8_SNORM;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB8_SNORM;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGBA8_SNORM;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SIGNED_NORMALIZED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COPY_READ_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.COPY_WRITE_BUFFER;

// Same as COPY_READ_BUFFER
/** @type {number} */
WebGL2RenderingContextBase.prototype.COPY_READ_BUFFER_BINDING;

// Same as COPY_WRITE_BUFFER
/** @type {number} */
WebGL2RenderingContextBase.prototype.COPY_WRITE_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BUFFER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BUFFER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BUFFER_START;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BUFFER_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_VERTEX_UNIFORM_BLOCKS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_FRAGMENT_UNIFORM_BLOCKS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_COMBINED_UNIFORM_BLOCKS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_UNIFORM_BUFFER_BINDINGS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_UNIFORM_BLOCK_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BUFFER_OFFSET_ALIGNMENT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.ACTIVE_UNIFORM_BLOCKS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_TYPE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BLOCK_INDEX;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_OFFSET;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_ARRAY_STRIDE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_MATRIX_STRIDE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_IS_ROW_MAJOR;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BLOCK_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BLOCK_DATA_SIZE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BLOCK_ACTIVE_UNIFORMS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER;

/** @type {number} */
WebGL2RenderingContextBase.prototype.INVALID_INDEX;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_VERTEX_OUTPUT_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_FRAGMENT_INPUT_COMPONENTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_SERVER_WAIT_TIMEOUT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.OBJECT_TYPE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SYNC_CONDITION;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SYNC_STATUS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SYNC_FLAGS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SYNC_FENCE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SYNC_GPU_COMMANDS_COMPLETE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.UNSIGNALED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SIGNALED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.ALREADY_SIGNALED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TIMEOUT_EXPIRED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.CONDITION_SATISFIED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.WAIT_FAILED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SYNC_FLUSH_COMMANDS_BIT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.VERTEX_ATTRIB_ARRAY_DIVISOR;

/** @type {number} */
WebGL2RenderingContextBase.prototype.ANY_SAMPLES_PASSED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.ANY_SAMPLES_PASSED_CONSERVATIVE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.SAMPLER_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.RGB10_A2UI;

/** @type {number} */
WebGL2RenderingContextBase.prototype.INT_2_10_10_10_REV;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_PAUSED;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_ACTIVE;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TRANSFORM_FEEDBACK_BINDING;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_IMMUTABLE_FORMAT;

/** @type {number} */
WebGL2RenderingContextBase.prototype.MAX_ELEMENT_INDEX;

/** @type {number} */
WebGL2RenderingContextBase.prototype.NUM_SAMPLE_COUNTS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TEXTURE_IMMUTABLE_LEVELS;

/** @type {number} */
WebGL2RenderingContextBase.prototype.TIMEOUT_IGNORED;

/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLQuery() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLSampler() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLSync() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLTransformFeedback() {}


/**
 * @constructor
 * @noalias
 * @extends {WebGLObject}
 */
function WebGLVertexArrayObject() {}


/**
 * @param {number} target
 * @param {WebGLQuery} query
 */
WebGL2RenderingContextBase.prototype.beginQuery = function(target, query) {};

/**
 * @param {number} primitiveMode
 */
WebGL2RenderingContextBase.prototype.beginTransformFeedback = function(primitiveMode) {};

/**
 * @param {number} target
 * @param {number} index
 * @param {WebGLBuffer} buffer
 */
WebGL2RenderingContextBase.prototype.bindBufferBase = function(target, index, buffer) {};

/**
 * @param {number} target
 * @param {number} index
 * @param {WebGLBuffer} buffer
 * @param {number} offset
 * @param {number} size
 */
WebGL2RenderingContextBase.prototype.bindBufferRange = function(target, index, buffer, offset, size) {};

/**
 * @param {number} unit
 * @param {WebGLSampler} sampler
 */
WebGL2RenderingContextBase.prototype.bindSampler = function(unit, sampler) {};

/**
 * @param {number} target
 * @param {WebGLTransformFeedback} id
 */
WebGL2RenderingContextBase.prototype.bindTransformFeedback = function(target, id) {};

/**
 * @param {WebGLVertexArrayObject} array
 */
WebGL2RenderingContextBase.prototype.bindVertexArray = function(array) {};

/**
 * @param {number} srcX0
 * @param {number} srcY0
 * @param {number} srcX1
 * @param {number} srcY1
 * @param {number} dstX0
 * @param {number} dstY0
 * @param {number} dstX1
 * @param {number} dstY1
 * @param {number} mask
 * @param {number} filter
 */
WebGL2RenderingContextBase.prototype.blitFramebuffer = function(
    srcX0, srcY0, srcX1, srcY1, dstX0, dstY0,
    dstX1, dstY1, mask, filter) {};

/**
 * @param {number} buffer
 * @param {number} drawbuffer
 * @param {Int32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.clearBufferiv = function(buffer, drawbuffer, value) {};

/**
 * @param {number} buffer
 * @param {number} drawbuffer
 * @param {Uint32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.clearBufferuiv = function(buffer, drawbuffer, value) {};

/**
 * @param {number} buffer
 * @param {number} drawbuffer
 * @param {Float32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.clearBufferfv = function(buffer, drawbuffer, value) {};

/**
 * @param {number} buffer
 * @param {number} drawbuffer
 * @param {number} depth
 * @param {number} stencil
 */
WebGL2RenderingContextBase.prototype.clearBufferfi = function(buffer, drawbuffer, depth, stencil) {};

/**
 * @param {WebGLSync} sync
 * @param {number} flags
 * @param {number} timeout
 * @return {number}
 */
WebGL2RenderingContextBase.prototype.clientWaitSync = function(sync, flags, timeout) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} internalformat
 * @param {number} width
 * @param {number} height
 * @param {number} depth
 * @param {number} border
 * @param {ArrayBufferView} data
 */
WebGL2RenderingContextBase.prototype.compressedTexImage3D = function(
    target, level, internalformat, width, height, depth, border, data) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} xoffset
 * @param {number} yoffset
 * @param {number} zoffset
 * @param {number} width
 * @param {number} height
 * @param {number} depth
 * @param {number} format
 * @param {ArrayBufferView} data
 */
WebGL2RenderingContextBase.prototype.compressedTexSubImage3D = function(
    target, level, xoffset, yoffset, zoffset, width, height, depth, format, data) {};

/**
 * @param {number} readTarget
 * @param {number} writeTarget
 * @param {number} readOffset
 * @param {number} writeOffset
 * @param {number} size
 */
WebGL2RenderingContextBase.prototype.copyBufferSubData = function(
    readTarget, writeTarget, readOffset, writeOffset, size) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} xoffset
 * @param {number} yoffset
 * @param {number} zoffset
 * @param {number} x
 * @param {number} y
 * @param {number} width
 * @param {number} height
 */
WebGL2RenderingContextBase.prototype.copyTexSubImage3D = function(
    target, level, xoffset, yoffset, zoffset, x, y, width, height) {};

/**
 * @return {!WebGLQuery}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.createQuery = function() {};

/**
 * @return {!WebGLSampler}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.createSampler = function() {};

/**
 * @return {!WebGLTransformFeedback}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.createTransformFeedback = function() {};

/**
 * @return {!WebGLVertexArrayObject}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.createVertexArray = function() {};


/**
 * @param {WebGLQuery} query
 */
WebGL2RenderingContextBase.prototype.deleteQuery = function(query) {};

/**
 * @param {WebGLSampler} sampler
 */
WebGL2RenderingContextBase.prototype.deleteSampler = function(sampler) {};

/**
 * @param {WebGLSync} sync
 */
WebGL2RenderingContextBase.prototype.deleteSync = function(sync) {};

/**
 * @param {WebGLTransformFeedback} feedback
 */
WebGL2RenderingContextBase.prototype.deleteTransformFeedback = function(feedback) {};

/**
 * @param {WebGLVertexArrayObject} vertexArray
 */
WebGL2RenderingContextBase.prototype.deleteVertexArray = function(vertexArray) {};

/**
 * @param {number} mode
 * @param {number} first
 * @param {number} count
 * @param {number} instanceCount
 */
WebGL2RenderingContextBase.prototype.drawArraysInstanced = function(mode, first, count, instanceCount) {};

/**
 * @param {number} mode
 * @param {number} count
 * @param {number} type
 * @param {number} offset
 * @param {number} instanceCount
 */
WebGL2RenderingContextBase.prototype.drawElementsInstanced = function(mode, count, type, offset, instanceCount) {};

/**
 * @param {number} mode
 * @param {number} start
 * @param {number} end
 * @param {number} count
 * @param {number} type
 * @param {number} offset
 */
WebGL2RenderingContextBase.prototype.drawRangeElements = function(mode, start, end, count, type, offset) {};

/**
 * @param {Array.<number>} buffers
 */
WebGL2RenderingContextBase.prototype.drawBuffers = function(buffers) {};

/**
 * @param {number} target
 */
WebGL2RenderingContextBase.prototype.endQuery = function(target) {};

/**
 */
WebGL2RenderingContextBase.prototype.endTransformFeedback = function() {};

/**
 * @param {number} condition
 * @param {number} flags
 * @return {WebGLSync}
 */
WebGL2RenderingContextBase.prototype.fenceSync = function(condition, flags) {};

/**
 * @param {number} target
 * @param {number} attachment
 * @param {WebGLTexture} texture
 * @param {number} level
 * @param {number} layer
 */
WebGL2RenderingContextBase.prototype.framebufferTextureLayer = function(
    target, attachment, texture, level, layer) {};

/**
 * @param {WebGLProgram} program
 * @param {number} uniformBlockIndex
 * @return {string}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getActiveUniformBlockName = function(program, uniformBlockIndex) {};

/**
 * @param {WebGLProgram} program
 * @param {number} uniformBlockIndex
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getActiveUniformBlockParameter = function(
    program, uniformBlockIndex, pname) {};

/**
 * @param {WebGLProgram} program
 * @param {Array.<number>} uniformIndices
 * @param {number} pname
 * @return {Array.<number>}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getActiveUniforms = function(
    program, uniformIndices, pname) {};

/**
 * @param {number} target
 * @param {number} offset
 * @param {ArrayBuffer} returnedData
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getBufferSubData = function(
    target, offset, returnedData) {};

/**
 * @param {WebGLProgram} program
 * @param {string} name
 * @return {number}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getFragDataLocation = function(program, name) {};

/**
 * @param {number} target
 * @param {number} index
 * @return {*}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getIndexedParameter = function(target, index) {};

/**
 * @param {number} target
 * @param {number} internalformat
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getInternalformatParameter = function(target, internalformat, pname) {};

/**
 * @param {number} target
 * @param {number} pname
 * @return {WebGLQuery}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getQuery = function(target, pname) {};

/**
 * @param {WebGLQuery} query
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getQueryParameter = function(query, pname) {};

/**
 * @param {WebGLSampler} sampler
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getSamplerParameter = function(sampler, pname) {};

/**
 * @param {WebGLSync} sync
 * @param {number} pname
 * @return {*}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getSyncParameter = function(sync, pname) {};

/**
 * @param {WebGLProgram} program
 * @param {number} index
 * @return {WebGLActiveInfo}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getTransformFeedbackVarying = function(program, index) {};

/**
 * @param {WebGLProgram} program
 * @param {string} uniformBlockName
 * @return {number}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getUniformBlockIndex = function(program, uniformBlockName) {};

/**
 * @param {WebGLProgram} program
 * @param {Array.<string>} uniformNames
 * @return {Array.<number>}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.getUniformIndices = function(program, uniformNames) {};

/**
 * @param {number} target
 * @param {Array.<number>} attachments
 */
WebGL2RenderingContextBase.prototype.invalidateFramebuffer = function(target, attachments) {};

/**
 * @param {number} target
 * @param {Array.<number>} attachments
 * @param {number} x
 * @param {number} y
 * @param {number} width
 * @param {number} height
 */
WebGL2RenderingContextBase.prototype.invalidateSubFramebuffer = function(
    target, attachments, x, y, width, height) {};

/**
 * @param {WebGLQuery} query
 * @return {boolean}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.isQuery = function(query) {};

/**
 * @param {WebGLSampler} sampler
 * @return {boolean}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.isSampler = function(sampler) {};

/**
 * @param {WebGLSync} sync
 * @return {boolean}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.isSync = function(sync) {};

/**
 * @param {WebGLTransformFeedback} feedback
 * @return {boolean}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.isTransformFeedback = function(feedback) {};

/**
 * @param {WebGLVertexArrayObject} vertexArray
 * @return {boolean}
 * @nosideeffects
 */
WebGL2RenderingContextBase.prototype.isVertexArray = function(vertexArray) {};

/**
 */
WebGL2RenderingContextBase.prototype.pauseTransformFeedback = function() {};

/**
 * @param {number} src
 */
WebGL2RenderingContextBase.prototype.readBuffer = function(src) {};

/**
 * @param {number} target
 * @param {number} samples
 * @param {number} internalformat
 * @param {number} width
 * @param {number} height
 */
WebGL2RenderingContextBase.prototype.renderbufferStorageMultisample = function(
    target, samples, internalformat, width, height) {};

/**
 */
WebGL2RenderingContextBase.prototype.resumeTransformFeedback = function() {};

/**
 * @param {WebGLSampler} sampler
 * @param {number} pname
 * @param {number} param
 */
WebGL2RenderingContextBase.prototype.samplerParameteri = function(sampler, pname, param) {};

/**
 * @param {WebGLSampler} sampler
 * @param {number} pname
 * @param {number} param
 */
WebGL2RenderingContextBase.prototype.samplerParameterf = function(sampler, pname, param) {};

/**
 * @param {number} target
 * @param {number} levels
 * @param {number} internalformat
 * @param {number} width
 * @param {number} height
 */
WebGL2RenderingContextBase.prototype.texStorage2D = function(target, levels, internalformat, width, height) {};

/**
 * @param {number} target
 * @param {number} levels
 * @param {number} internalformat
 * @param {number} width
 * @param {number} height
 * @param {number} depth
 */
WebGL2RenderingContextBase.prototype.texStorage3D = function(target, levels, internalformat, width, height, depth) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} internalformat
 * @param {number} width
 * @param {number} height
 * @param {number} depth
 * @param {number} border
 * @param {number} format
 * @param {number} type
 * @param {ArrayBufferView} pixels
 */
WebGL2RenderingContextBase.prototype.texImage3D = function(
    target, level, internalformat, width, height, depth, border, format, type, pixels) {};

/**
 * @param {number} target
 * @param {number} level
 * @param {number} xoffset
 * @param {number} yoffset
 * @param {number} zoffset
 * @param {number} format or width
 * @param {number} type or height
 * @param {ImageData|HTMLImageElement|HTMLCanvasElement|HTMLVideoElement|
 *     number} source or depth
 * @param {number=} opt_format
 * @param {number=} opt_type
 * @param {ArrayBufferView=} opt_pixels
 */
WebGL2RenderingContextBase.prototype.texSubImage3D = function(
    target, level, xoffset, yoffset, zoffset, format, type, source, opt_format, opt_type, opt_pixels) {};

/**
 * @param {WebGLProgram} program
 * @param {Array.<string>} varyings
 * @param {number} bufferMode
 */
WebGL2RenderingContextBase.prototype.transformFeedbackVaryings = function(program, varyings, bufferMode) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} v0
 */
WebGL2RenderingContextBase.prototype.uniform1ui = function(location, v0) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} v0
 * @param {number} v1
 */
WebGL2RenderingContextBase.prototype.uniform2ui = function(location, v0, v1) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} v0
 * @param {number} v1
 * @param {number} v2
 */
WebGL2RenderingContextBase.prototype.uniform3ui = function(location, v0, v1, v2) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {number} v0
 * @param {number} v1
 * @param {number} v2
 * @param {number} v3
 */
WebGL2RenderingContextBase.prototype.uniform4ui = function(location, v0, v1, v2, v3) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Uint32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniform1uiv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Uint32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniform2uiv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Uint32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniform3uiv = function(location, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {Uint32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniform4uiv = function(location, value) {};

/**
 * @param {WebGLProgram} program
 * @param {number} uniformBlockIndex
 * @param {number} uniformBlockBinding
 */
WebGL2RenderingContextBase.prototype.uniformBlockBinding = function(program, uniformBlockIndex, uniformBlockBinding) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniformMatrix2x3fv = function(location, transpose, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniformMatrix3x2fv = function(location, transpose, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniformMatrix2x4fv = function(location, transpose, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniformMatrix4x2fv = function(location, transpose, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniformMatrix3x4fv = function(location, transpose, value) {};

/**
 * @param {WebGLUniformLocation} location
 * @param {boolean} transpose
 * @param {Float32Array|Array.<number>} value
 */
WebGL2RenderingContextBase.prototype.uniformMatrix4x3fv = function(location, transpose, value) {};

/**
 * @param {number} index
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @param {number} w
 */
WebGL2RenderingContextBase.prototype.vertexAttribI4i = function(index, x, y, z, w) {};

/**
 * @param {number} index
 * @param {(Array.<number>|Int32Array)} v
 */
WebGL2RenderingContextBase.prototype.vertexAttribI4iv = function(index, v) {};

/**
 * @param {number} index
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @param {number} w
 */
WebGL2RenderingContextBase.prototype.vertexAttribI4ui = function(index, x, y, z, w) {};

/**
 * @param {number} index
 * @param {(Array.<number>|Uint32Array)} v
 */
WebGL2RenderingContextBase.prototype.vertexAttribI4uiv = function(index, v) {};

/**
 * @param {number} index
 * @param {number} size
 * @param {number} type
 * @param {number} stride
 * @param {number} offset
 */
WebGL2RenderingContextBase.prototype.vertexAttribIPointer = function(index, size, type, stride, offset) {};

/**
 * @param {number} index
 * @param {number} divisor
 */
WebGL2RenderingContextBase.prototype.vertexAttribDivisor = function(index, divisor) {};

/**
 * @param {WebGLSync} sync
 * @param {number} flags
 * @param {number} timeout
 */
WebGL2RenderingContextBase.prototype.waitSync = function(sync, flags, timeout) {};

/**
 * @constructor
 * @noalias
 * @extends {WebGL2RenderingContextBase}
 */
function WebGL2RenderingContext() {}
