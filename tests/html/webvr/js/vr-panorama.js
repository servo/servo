// Copyright 2016 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/* global mat4, WGLUProgram */

window.VRPanorama = (function () {
  "use strict";

  var panoVS = [
    "uniform mat4 projectionMat;",
    "uniform mat4 modelViewMat;",
    "attribute vec3 position;",
    "attribute vec2 texCoord;",
    "varying vec2 vTexCoord;",

    "void main() {",
    "  vTexCoord = texCoord;",
    "  gl_Position = projectionMat * modelViewMat * vec4( position, 1.0 );",
    "}",
  ].join("\n");

  var panoFS = [
    "precision mediump float;",
    "uniform sampler2D diffuse;",
    "varying vec2 vTexCoord;",

    "void main() {",
    "  gl_FragColor = texture2D(diffuse, vTexCoord);",
    "}",
  ].join("\n");

  var Panorama = function (gl) {
    this.gl = gl;

    this.texture = gl.createTexture();

    this.program = new WGLUProgram(gl);
    this.program.attachShaderSource(panoVS, gl.VERTEX_SHADER);
    this.program.attachShaderSource(panoFS, gl.FRAGMENT_SHADER);
    this.program.bindAttribLocation({
      position: 0,
      texCoord: 1
    });
    this.program.link();

    var panoVerts = [];
    var panoIndices = [];

    var radius = 2; // 2 meter radius sphere
    var latSegments = 40;
    var lonSegments = 40;

    // Create the vertices
    for (var i=0; i <= latSegments; ++i) {
      var theta = i * Math.PI / latSegments;
      var sinTheta = Math.sin(theta);
      var cosTheta = Math.cos(theta);

      for (var j=0; j <= lonSegments; ++j) {
        var phi = j * 2 * Math.PI / lonSegments;
        var sinPhi = Math.sin(phi);
        var cosPhi = Math.cos(phi);

        var x = sinPhi * sinTheta;
        var y = cosTheta;
        var z = -cosPhi * sinTheta;
        var u = (j / lonSegments);
        var v = (i / latSegments);

        panoVerts.push(x * radius, y * radius, z * radius, u, v);
      }
    }

    // Create the indices
    for (var i = 0; i < latSegments; ++i) {
      var offset0 = i * (lonSegments+1);
      var offset1 = (i+1) * (lonSegments+1);
      for (var j = 0; j < lonSegments; ++j) {
        var index0 = offset0+j;
        var index1 = offset1+j;
        panoIndices.push(
          index0, index1, index0+1,
          index1, index1+1, index0+1
        );
      }
    }

    this.vertBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.vertBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(panoVerts), gl.STATIC_DRAW);

    this.indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(panoIndices), gl.STATIC_DRAW);

    this.indexCount = panoIndices.length;

    this.imgElement = null;
    this.videoElement = null;
  };

  Panorama.prototype.setImage = function (url) {
    var gl = this.gl;
    var self = this;

    return new Promise(function(resolve, reject) {
      var img = new Image();
      img.addEventListener('load', function() {
        self.imgElement = img;
        self.videoElement = null;

        gl.bindTexture(gl.TEXTURE_2D, self.texture);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, gl.RGB, gl.UNSIGNED_BYTE, img);

        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

        resolve(self.imgElement);
      });
      img.addEventListener('error', function(ev) {
        console.error(ev.message);
        reject(ev.message);
      }, false);
      img.crossOrigin = 'anonymous';
      img.src = url;
    });
  };

  Panorama.prototype.setVideo = function (url) {
    var gl = this.gl;
    var self = this;

    return new Promise(function(resolve, reject) {
      var video = document.createElement('video');
      video.addEventListener('canplay', function() {
        // Added "click to play" UI?
      });

      video.addEventListener('playing', function() {
        self.videoElement = video;
        self.imgElement = null;

        gl.bindTexture(gl.TEXTURE_2D, self.texture);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, gl.RGB, gl.UNSIGNED_BYTE, self.videoElement);

        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

        resolve(self.videoElement);
      });

      video.addEventListener('error', function(ev) {
        console.error(video.error);
        reject(video.error);
      }, false);

      video.loop = true;
      video.autoplay = true;
      video.crossOrigin = 'anonymous';
      video.setAttribute('webkit-playsinline', '');
      video.src = url;
    });
  };

  Panorama.prototype.play = function() {
    if (this.videoElement)
      this.videoElement.play();
  };

  Panorama.prototype.pause = function() {
    if (this.videoElement)
      this.videoElement.pause();
  };

  Panorama.prototype.isPaused = function() {
    if (this.videoElement)
      return this.videoElement.paused;
    return false;
  };

  Panorama.prototype.render = function (projectionMat, modelViewMat) {
    var gl = this.gl;
    var program = this.program;

    if (!this.imgElement && !this.videoElement)
      return;

    program.use();

    gl.uniformMatrix4fv(program.uniform.projectionMat, false, projectionMat);
    gl.uniformMatrix4fv(program.uniform.modelViewMat, false, modelViewMat);

    gl.bindBuffer(gl.ARRAY_BUFFER, this.vertBuffer);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);

    gl.enableVertexAttribArray(program.attrib.position);
    gl.enableVertexAttribArray(program.attrib.texCoord);

    gl.vertexAttribPointer(program.attrib.position, 3, gl.FLOAT, false, 20, 0);
    gl.vertexAttribPointer(program.attrib.texCoord, 2, gl.FLOAT, false, 20, 12);

    gl.activeTexture(gl.TEXTURE0);
    gl.uniform1i(this.program.uniform.diffuse, 0);
    gl.bindTexture(gl.TEXTURE_2D, this.texture);

    if (this.videoElement && !this.videoElement.paused) {
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, gl.RGB, gl.UNSIGNED_BYTE, this.videoElement);
    }

    gl.drawElements(gl.TRIANGLES, this.indexCount, gl.UNSIGNED_SHORT, 0);
  };

  return Panorama;
})();
