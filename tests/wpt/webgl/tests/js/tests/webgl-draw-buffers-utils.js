/*
** Copyright (c) 2017 The Khronos Group Inc.
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

// This file contains utilities shared between tests for the WEBGL_draw_buffers extension and multiple draw buffers functionality in WebGL 2.0.

'use strict';

var WebGLDrawBuffersUtils = function(gl, ext) {

  var getMaxUsableColorAttachments = function() {
    var maxDrawingBuffers;
    var maxColorAttachments;
    if (ext) {
      // EXT_draw_buffers
      maxDrawingBuffers = gl.getParameter(ext.MAX_DRAW_BUFFERS_WEBGL);
      maxColorAttachments = gl.getParameter(ext.MAX_COLOR_ATTACHMENTS_WEBGL);
    } else {
      // WebGL 2.0
      maxDrawingBuffers = gl.getParameter(gl.MAX_DRAW_BUFFERS);
      maxColorAttachments = gl.getParameter(gl.MAX_COLOR_ATTACHMENTS);
    }
    var maxUniformVectors = gl.getParameter(gl.MAX_FRAGMENT_UNIFORM_VECTORS);
    return Math.min(maxDrawingBuffers, maxColorAttachments, maxUniformVectors);
  };

  var makeColorAttachmentArray = function(size) {
    var array = []
    for (var ii = 0; ii < size; ++ii) {
      array.push(gl.COLOR_ATTACHMENT0 + ii);
    }
    return array;
  }

  var checkProgram = wtu.setupTexturedQuad(gl);

  var checkAttachmentsForColorFn = function(attachments, colorFn) {
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.useProgram(checkProgram);
    attachments.forEach(function(attachment, index) {
      gl.bindTexture(gl.TEXTURE_2D, attachment.texture);
      wtu.clearAndDrawUnitQuad(gl);
      var expectedColor = colorFn(attachment, index);
      var tolerance = 0;
      expectedColor.forEach(function(v) {
        if (v != 0 && v != 255) {
          tolerance = 8;
        }
      });
      wtu.checkCanvas(gl, expectedColor, "attachment " + index + " should be " + expectedColor.toString(), tolerance);
    });
    debug("");
  };

  var checkAttachmentsForColor = function(attachments, color) {
    checkAttachmentsForColorFn(attachments, function(attachment, index) {
      return color || attachment.color;
    });
  };

  return {
    getMaxUsableColorAttachments: getMaxUsableColorAttachments,
    makeColorAttachmentArray: makeColorAttachmentArray,
    checkAttachmentsForColorFn: checkAttachmentsForColorFn,
    checkAttachmentsForColor: checkAttachmentsForColor
  };
};
