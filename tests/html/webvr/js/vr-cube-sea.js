// Copyright 2016 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

/* global mat4, WGLUProgram */

window.VRCubeSea = (function () {
  "use strict";

  var cubeSeaVS = [
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

  var cubeSeaFS = [
    "precision mediump float;",
    "uniform sampler2D diffuse;",
    "varying vec2 vTexCoord;",

    "void main() {",
    "  gl_FragColor = texture2D(diffuse, vTexCoord);",
    "}",
  ].join("\n");

  var CubeSea = function (gl, texture) {
    this.gl = gl;

    this.statsMat = mat4.create();

    this.texture = texture;

    this.program = new WGLUProgram(gl);
    this.program.attachShaderSource(cubeSeaVS, gl.VERTEX_SHADER);
    this.program.attachShaderSource(cubeSeaFS, gl.FRAGMENT_SHADER);
    this.program.bindAttribLocation({
      position: 0,
      texCoord: 1
    });
    this.program.link();

    var cubeVerts = [];
    var cubeIndices = [];

    // Build a single cube.
    function appendCube (x, y, z) {
      if (!x && !y && !z) {
        // Don't create a cube in the center.
        return;
      }

      var size = 0.2;
      // Bottom
      var idx = cubeVerts.length / 5.0;
      cubeIndices.push(idx, idx + 1, idx + 2);
      cubeIndices.push(idx, idx + 2, idx + 3);

      cubeVerts.push(x - size, y - size, z - size, 0.0, 1.0);
      cubeVerts.push(x + size, y - size, z - size, 1.0, 1.0);
      cubeVerts.push(x + size, y - size, z + size, 1.0, 0.0);
      cubeVerts.push(x - size, y - size, z + size, 0.0, 0.0);

      // Top
      idx = cubeVerts.length / 5.0;
      cubeIndices.push(idx, idx + 2, idx + 1);
      cubeIndices.push(idx, idx + 3, idx + 2);

      cubeVerts.push(x - size, y + size, z - size, 0.0, 0.0);
      cubeVerts.push(x + size, y + size, z - size, 1.0, 0.0);
      cubeVerts.push(x + size, y + size, z + size, 1.0, 1.0);
      cubeVerts.push(x - size, y + size, z + size, 0.0, 1.0);

      // Left
      idx = cubeVerts.length / 5.0;
      cubeIndices.push(idx, idx + 2, idx + 1);
      cubeIndices.push(idx, idx + 3, idx + 2);

      cubeVerts.push(x - size, y - size, z - size, 0.0, 1.0);
      cubeVerts.push(x - size, y + size, z - size, 0.0, 0.0);
      cubeVerts.push(x - size, y + size, z + size, 1.0, 0.0);
      cubeVerts.push(x - size, y - size, z + size, 1.0, 1.0);

      // Right
      idx = cubeVerts.length / 5.0;
      cubeIndices.push(idx, idx + 1, idx + 2);
      cubeIndices.push(idx, idx + 2, idx + 3);

      cubeVerts.push(x + size, y - size, z - size, 1.0, 1.0);
      cubeVerts.push(x + size, y + size, z - size, 1.0, 0.0);
      cubeVerts.push(x + size, y + size, z + size, 0.0, 0.0);
      cubeVerts.push(x + size, y - size, z + size, 0.0, 1.0);

      // Back
      idx = cubeVerts.length / 5.0;
      cubeIndices.push(idx, idx + 2, idx + 1);
      cubeIndices.push(idx, idx + 3, idx + 2);

      cubeVerts.push(x - size, y - size, z - size, 1.0, 1.0);
      cubeVerts.push(x + size, y - size, z - size, 0.0, 1.0);
      cubeVerts.push(x + size, y + size, z - size, 0.0, 0.0);
      cubeVerts.push(x - size, y + size, z - size, 1.0, 0.0);

      // Front
      idx = cubeVerts.length / 5.0;
      cubeIndices.push(idx, idx + 1, idx + 2);
      cubeIndices.push(idx, idx + 2, idx + 3);

      cubeVerts.push(x - size, y - size, z + size, 0.0, 1.0);
      cubeVerts.push(x + size, y - size, z + size, 1.0, 1.0);
      cubeVerts.push(x + size, y + size, z + size, 1.0, 0.0);
      cubeVerts.push(x - size, y + size, z + size, 0.0, 0.0);
    }

    var gridSize = 10;

    // Build the cube sea
    for (var x = 0; x < gridSize; ++x) {
      for (var y = 0; y < gridSize; ++y) {
        for (var z = 0; z < gridSize; ++z) {
          appendCube(x - (gridSize / 2), y - (gridSize / 2), z - (gridSize / 2));
        }
      }
    }

    this.vertBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.vertBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(cubeVerts), gl.STATIC_DRAW);

    this.indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(cubeIndices), gl.STATIC_DRAW);

    this.indexCount = cubeIndices.length;
  };

  var mortimer = mat4.create();

  var a = [0.9868122935295105, -0.03754837438464165, -0.15745431184768677, 0, 0.011360996402800083, 0.9863911271095276, -0.1640235036611557, 0, 0.16147033870220184, 0.16007155179977417, 0.9738093614578247, 0, 0.192538782954216, 0.024526841938495636, -0.001076754298992455, 1.0000001192092896];
  for (var i = 0; i < 16; ++i) {
    mortimer[i] = a[i];
  }

  CubeSea.prototype.render = function (projectionMat, modelViewMat, stats) {
    var gl = this.gl;
    var program = this.program;

    //mat4.invert(mortimer, modelViewMat);

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

    gl.drawElements(gl.TRIANGLES, this.indexCount, gl.UNSIGNED_SHORT, 0);

    if (stats) {
      // To ensure that the FPS counter is visible in VR mode we have to
      // render it as part of the scene.
      mat4.fromTranslation(this.statsMat, [0, -0.3, -0.5]);
      mat4.scale(this.statsMat, this.statsMat, [0.3, 0.3, 0.3]);
      mat4.rotateX(this.statsMat, this.statsMat, -0.75);
      mat4.multiply(this.statsMat, modelViewMat, this.statsMat);
      stats.render(projectionMat, this.statsMat);
    }
  };

  return CubeSea;
})();
