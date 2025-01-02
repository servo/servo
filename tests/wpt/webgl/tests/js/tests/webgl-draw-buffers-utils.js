/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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
