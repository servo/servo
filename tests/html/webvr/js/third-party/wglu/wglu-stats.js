/*
Copyright (c) 2016, Brandon Jones.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
*/

/*
Heavily inspired by Mr. Doobs stats.js, this FPS counter is rendered completely
with WebGL, allowing it to be shown in cases where overlaid HTML elements aren't
usable (like WebVR), or if you want the FPS counter to be rendered as part of
your scene.

See stats-test.html for basic usage.
*/
var WGLUStats = (function() {

  "use strict";

  //--------------------
  // glMatrix functions
  //--------------------

  // These functions have been copied here from glMatrix (glmatrix.net) to allow
  // this file to run standalone.

  var mat4_identity = function(out) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = 1;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = 1;
      out[11] = 0;
      out[12] = 0;
      out[13] = 0;
      out[14] = 0;
      out[15] = 1;
      return out;
  };

  var mat4_multiply = function (out, a, b) {
      var a00 = a[0], a01 = a[1], a02 = a[2], a03 = a[3],
          a10 = a[4], a11 = a[5], a12 = a[6], a13 = a[7],
          a20 = a[8], a21 = a[9], a22 = a[10], a23 = a[11],
          a30 = a[12], a31 = a[13], a32 = a[14], a33 = a[15];

      // Cache only the current line of the second matrix
      var b0  = b[0], b1 = b[1], b2 = b[2], b3 = b[3];
      out[0] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
      out[1] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
      out[2] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
      out[3] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

      b0 = b[4]; b1 = b[5]; b2 = b[6]; b3 = b[7];
      out[4] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
      out[5] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
      out[6] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
      out[7] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

      b0 = b[8]; b1 = b[9]; b2 = b[10]; b3 = b[11];
      out[8] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
      out[9] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
      out[10] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
      out[11] = b0*a03 + b1*a13 + b2*a23 + b3*a33;

      b0 = b[12]; b1 = b[13]; b2 = b[14]; b3 = b[15];
      out[12] = b0*a00 + b1*a10 + b2*a20 + b3*a30;
      out[13] = b0*a01 + b1*a11 + b2*a21 + b3*a31;
      out[14] = b0*a02 + b1*a12 + b2*a22 + b3*a32;
      out[15] = b0*a03 + b1*a13 + b2*a23 + b3*a33;
      return out;
  };

  var mat4_fromTranslation = function(out, v) {
      out[0] = 1;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = 1;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = 1;
      out[11] = 0;
      out[12] = v[0];
      out[13] = v[1];
      out[14] = v[2];
      out[15] = 1;
      return out;
  };

  var mat4_ortho = function (out, left, right, bottom, top, near, far) {
      var lr = 1 / (left - right),
          bt = 1 / (bottom - top),
          nf = 1 / (near - far);
      out[0] = -2 * lr;
      out[1] = 0;
      out[2] = 0;
      out[3] = 0;
      out[4] = 0;
      out[5] = -2 * bt;
      out[6] = 0;
      out[7] = 0;
      out[8] = 0;
      out[9] = 0;
      out[10] = 2 * nf;
      out[11] = 0;
      out[12] = (left + right) * lr;
      out[13] = (top + bottom) * bt;
      out[14] = (far + near) * nf;
      out[15] = 1;
      return out;
  };

  var mat4_translate = function (out, a, v) {
      var x = v[0], y = v[1], z = v[2],
          a00, a01, a02, a03,
          a10, a11, a12, a13,
          a20, a21, a22, a23;

      if (a === out) {
          out[12] = a[0] * x + a[4] * y + a[8] * z + a[12];
          out[13] = a[1] * x + a[5] * y + a[9] * z + a[13];
          out[14] = a[2] * x + a[6] * y + a[10] * z + a[14];
          out[15] = a[3] * x + a[7] * y + a[11] * z + a[15];
      } else {
          a00 = a[0]; a01 = a[1]; a02 = a[2]; a03 = a[3];
          a10 = a[4]; a11 = a[5]; a12 = a[6]; a13 = a[7];
          a20 = a[8]; a21 = a[9]; a22 = a[10]; a23 = a[11];

          out[0] = a00; out[1] = a01; out[2] = a02; out[3] = a03;
          out[4] = a10; out[5] = a11; out[6] = a12; out[7] = a13;
          out[8] = a20; out[9] = a21; out[10] = a22; out[11] = a23;

          out[12] = a00 * x + a10 * y + a20 * z + a[12];
          out[13] = a01 * x + a11 * y + a21 * z + a[13];
          out[14] = a02 * x + a12 * y + a22 * z + a[14];
          out[15] = a03 * x + a13 * y + a23 * z + a[15];
      }

      return out;
  };

  var mat4_scale = function(out, a, v) {
      var x = v[0], y = v[1], z = v[2];

      out[0] = a[0] * x;
      out[1] = a[1] * x;
      out[2] = a[2] * x;
      out[3] = a[3] * x;
      out[4] = a[4] * y;
      out[5] = a[5] * y;
      out[6] = a[6] * y;
      out[7] = a[7] * y;
      out[8] = a[8] * z;
      out[9] = a[9] * z;
      out[10] = a[10] * z;
      out[11] = a[11] * z;
      out[12] = a[12];
      out[13] = a[13];
      out[14] = a[14];
      out[15] = a[15];
      return out;
  };

  //-------------------
  // Utility functions
  //-------------------

  function linkProgram(gl, vertexSource, fragmentSource, attribLocationMap) {
    // No error checking for brevity.
    var vertexShader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vertexShader, vertexSource);
    gl.compileShader(vertexShader);

    var fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fragmentShader, fragmentSource);
    gl.compileShader(fragmentShader);

    var program = gl.createProgram();
    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);

    for (var attribName in attribLocationMap)
      gl.bindAttribLocation(program, attribLocationMap[attribName], attribName);

    gl.linkProgram(program);

    gl.deleteShader(vertexShader);
    gl.deleteShader(fragmentShader);

    return program;
  }

  function getProgramUniforms(gl, program) {
    var uniforms = {};
    var uniformCount = gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS);
    var uniformName = "";
    for (var i = 0; i < uniformCount; i++) {
      var uniformInfo = gl.getActiveUniform(program, i);
      uniformName = uniformInfo.name.replace("[0]", "");
      uniforms[uniformName] = gl.getUniformLocation(program, uniformName);
    }
    return uniforms;
  }

  //----------------------------
  // Seven-segment text display
  //----------------------------

  var sevenSegmentVS = [
    "uniform mat4 projectionMat;",
    "uniform mat4 modelViewMat;",
    "attribute vec2 position;",

    "void main() {",
    "  gl_Position = projectionMat * modelViewMat * vec4( position, 0.0, 1.0 );",
    "}",
  ].join("\n");

  var sevenSegmentFS = [
    "precision mediump float;",
    "uniform vec4 color;",

    "void main() {",
    "  gl_FragColor = color;",
    "}",
  ].join("\n");

  var SevenSegmentText = function (gl) {
    this.gl = gl;

    this.attribs = {
      position: 0,
      color: 1
    };

    this.program = linkProgram(gl, sevenSegmentVS, sevenSegmentFS, this.attribs);
    this.uniforms = getProgramUniforms(gl, this.program);

    var verts = [];
    var segmentIndices = {};
    var indices = [];

    var width = 0.5;
    var thickness = 0.25;
    this.kerning = 2.0;

    this.matrix = new Float32Array(16);

    function defineSegment(id, left, top, right, bottom) {
      var idx = verts.length / 2;
      verts.push(
          left, top,
          right, top,
          right, bottom,
          left, bottom);

      segmentIndices[id] = [
          idx, idx+2, idx+1,
          idx, idx+3, idx+2];
    }

    var characters = {};
    this.characters = characters;

    function defineCharacter(c, segments) {
      var character = {
        character: c,
        offset: indices.length * 2,
        count: 0
      };

      for (var i = 0; i < segments.length; ++i) {
        var idx = segments[i];
        var segment = segmentIndices[idx];
        character.count += segment.length;
        indices.push.apply(indices, segment);
      }

      characters[c] = character;
    }

    /* Segment layout is as follows:

    |-0-|
    3   4
    |-1-|
    5   6
    |-2-|

    */

    defineSegment(0, -1, 1, width, 1-thickness);
    defineSegment(1, -1, thickness*0.5, width, -thickness*0.5);
    defineSegment(2, -1, -1+thickness, width, -1);
    defineSegment(3, -1, 1, -1+thickness, -thickness*0.5);
    defineSegment(4, width-thickness, 1, width, -thickness*0.5);
    defineSegment(5, -1, thickness*0.5, -1+thickness, -1);
    defineSegment(6, width-thickness, thickness*0.5, width, -1);


    defineCharacter("0", [0, 2, 3, 4, 5, 6]);
    defineCharacter("1", [4, 6]);
    defineCharacter("2", [0, 1, 2, 4, 5]);
    defineCharacter("3", [0, 1, 2, 4, 6]);
    defineCharacter("4", [1, 3, 4, 6]);
    defineCharacter("5", [0, 1, 2, 3, 6]);
    defineCharacter("6", [0, 1, 2, 3, 5, 6]);
    defineCharacter("7", [0, 4, 6]);
    defineCharacter("8", [0, 1, 2, 3, 4, 5, 6]);
    defineCharacter("9", [0, 1, 2, 3, 4, 6]);
    defineCharacter("A", [0, 1, 3, 4, 5, 6]);
    defineCharacter("B", [1, 2, 3, 5, 6]);
    defineCharacter("C", [0, 2, 3, 5]);
    defineCharacter("D", [1, 2, 4, 5, 6]);
    defineCharacter("E", [0, 1, 2, 4, 6]);
    defineCharacter("F", [0, 1, 3, 5]);
    defineCharacter("P", [0, 1, 3, 4, 5]);
    defineCharacter("-", [1]);
    defineCharacter(" ", []);
    defineCharacter("_", [2]); // Used for undefined characters

    this.vertBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.vertBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(verts), gl.DYNAMIC_DRAW);

    this.indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), gl.STATIC_DRAW);
  };

  SevenSegmentText.prototype.render = function(projectionMat, modelViewMat, text, r, g, b, a) {
    var gl = this.gl;

    if (r == undefined || g == undefined || b == undefined) {
      r = 0.0;
      g = 1.0;
      b = 0.0;
    }

    if (a == undefined)
      a = 1.0;

    gl.useProgram(this.program);

    gl.uniformMatrix4fv(this.uniforms.projectionMat, false, projectionMat);
    gl.uniform4f(this.uniforms.color, r, g, b, a);

    gl.bindBuffer(gl.ARRAY_BUFFER, this.vertBuffer);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.indexBuffer);

    gl.enableVertexAttribArray(this.attribs.position);
    gl.vertexAttribPointer(this.attribs.position, 2, gl.FLOAT, false, 8, 0);

    text = text.toUpperCase();

    var offset = 0;

    for (var i = 0; i < text.length; ++i) {
      var c;
      if (text[i] in this.characters) {
        c = this.characters[text[i]];
      } else {
        c = this.characters["_"];
      }

      if (c.count != 0) {
        mat4_fromTranslation(this.matrix, [offset, 0, 0]);
        mat4_multiply(this.matrix, modelViewMat, this.matrix);

        gl.uniformMatrix4fv(this.uniforms.modelViewMat, false, this.matrix);
        gl.drawElements(gl.TRIANGLES, c.count, gl.UNSIGNED_SHORT, c.offset);

      }

      offset += this.kerning;
    }
  }

  //-----------
  // FPS Graph
  //-----------

  var statsVS = [
    "uniform mat4 projectionMat;",
    "uniform mat4 modelViewMat;",
    "attribute vec3 position;",
    "attribute vec3 color;",
    "varying vec4 vColor;",

    "void main() {",
    "  vColor = vec4(color, 1.0);",
    "  gl_Position = projectionMat * modelViewMat * vec4( position, 1.0 );",
    "}",
  ].join("\n");

  var statsFS = [
    "precision mediump float;",
    "varying vec4 vColor;",

    "void main() {",
    "  gl_FragColor = vColor;",
    "}",
  ].join("\n");

  var segments = 30;
  var maxFPS = 90;

  function segmentToX(i) {
    return ((0.9/segments) * i) - 0.45;
  }

  function fpsToY(value) {
    return (Math.min(value, maxFPS) * (0.7 / maxFPS)) - 0.45;
  }

  function fpsToRGB(value) {
    return {
      r: Math.max(0.0, Math.min(1.0, 1.0 - (value/60))),
      g: Math.max(0.0, Math.min(1.0, ((value-15)/(maxFPS-15)))),
      b: Math.max(0.0, Math.min(1.0, ((value-15)/(maxFPS-15))))
    };
  }

  var now = /*( performance && performance.now ) ? performance.now.bind( performance ) :*/ Date.now;

  var Stats = function(gl) {
    this.gl = gl;

    this.sevenSegmentText = new SevenSegmentText(gl);

    this.startTime = now();
    this.prevTime = this.startTime;
    this.frames = 0;
    this.fps = 0;

    this.orthoProjMatrix = new Float32Array(16);
    this.orthoViewMatrix = new Float32Array(16);
    this.modelViewMatrix = new Float32Array(16);

    // Hard coded because it doesn't change:
    // Scale by 0.075 in X and Y
    // Translate into upper left corner w/ z = 0.02
    this.textMatrix = new Float32Array([
      0.075, 0, 0, 0,
      0, 0.075, 0, 0,
      0, 0, 1, 0,
      -0.3625, 0.3625, 0.02, 1
    ]);

    this.lastSegment = 0;

    this.attribs = {
      position: 0,
      color: 1
    };

    this.program = linkProgram(gl, statsVS, statsFS, this.attribs);
    this.uniforms = getProgramUniforms(gl, this.program);

    var fpsVerts = [];
    var fpsIndices = [];

    // Graph geometry
    for (var i = 0; i < segments; ++i) {
      // Bar top
      fpsVerts.push(segmentToX(i), fpsToY(0), 0.02, 0.0, 1.0, 1.0);
      fpsVerts.push(segmentToX(i+1), fpsToY(0), 0.02, 0.0, 1.0, 1.0);

      // Bar bottom
      fpsVerts.push(segmentToX(i), fpsToY(0), 0.02, 0.0, 1.0, 1.0);
      fpsVerts.push(segmentToX(i+1), fpsToY(0), 0.02, 0.0, 1.0, 1.0);

      var idx = i * 4;
      fpsIndices.push(idx, idx+3, idx+1,
                      idx+3, idx, idx+2);
    }

    function addBGSquare(left, bottom, right, top, z, r, g, b) {
      var idx = fpsVerts.length / 6;

      fpsVerts.push(left, bottom, z, r, g, b);
      fpsVerts.push(right, top, z, r, g, b);
      fpsVerts.push(left, top, z, r, g, b);
      fpsVerts.push(right, bottom, z, r, g, b);

      fpsIndices.push(idx, idx+1, idx+2,
                     idx, idx+3, idx+1);
    };

    // Panel Background
    addBGSquare(-0.5, -0.5, 0.5, 0.5, 0.0, 0.0, 0.0, 0.125);

    // FPS Background
    addBGSquare(-0.45, -0.45, 0.45, 0.25, 0.01, 0.0, 0.0, 0.4);

    // 30 FPS line
    addBGSquare(-0.45, fpsToY(30), 0.45, fpsToY(32), 0.015, 0.5, 0.0, 0.5);

    // 60 FPS line
    addBGSquare(-0.45, fpsToY(60), 0.45, fpsToY(62), 0.015, 0.2, 0.0, 0.75);

    this.fpsVertBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.fpsVertBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(fpsVerts), gl.DYNAMIC_DRAW);

    this.fpsIndexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.fpsIndexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(fpsIndices), gl.STATIC_DRAW);

    this.fpsIndexCount = fpsIndices.length;
  };

  Stats.prototype.begin = function() {
    this.startTime = now();
  };

  Stats.prototype.end = function() {
    var time = now();

    this.frames++;

    if (time > this.prevTime + 250) {
      this.fps = Math.round((this.frames * 1000) / (time - this.prevTime));

      this.updateGraph(this.fps);

      this.prevTime = time;
      this.frames = 0;
    }
  };

  Stats.prototype.updateGraph = function(value) {
    var gl = this.gl;

    var color = fpsToRGB(value);

    gl.bindBuffer(gl.ARRAY_BUFFER, this.fpsVertBuffer);

    // Update the current segment with the new FPS value
    var updateVerts = [
      segmentToX(this.lastSegment), fpsToY(value), 0.02, color.r, color.g, color.b,
      segmentToX(this.lastSegment+1), fpsToY(value), 0.02, color.r, color.g, color.b,
      segmentToX(this.lastSegment), fpsToY(0), 0.02, color.r, color.g, color.b,
      segmentToX(this.lastSegment+1), fpsToY(0), 0.02, color.r, color.g, color.b,
    ];

    // Re-shape the next segment into the green "progress" line
    color.r = 0.2;
    color.g = 1.0;
    color.b = 0.2;

    if (this.lastSegment == segments - 1) {
      // If we're updating the last segment we need to do two bufferSubDatas
      // to update the segment and turn the first segment into the progress line.
      gl.bufferSubData(gl.ARRAY_BUFFER, this.lastSegment * 24 * 4, new Float32Array(updateVerts));
      updateVerts = [
        segmentToX(0), fpsToY(maxFPS), 0.02, color.r, color.g, color.b,
        segmentToX(.25), fpsToY(maxFPS), 0.02, color.r, color.g, color.b,
        segmentToX(0), fpsToY(0), 0.02, color.r, color.g, color.b,
        segmentToX(.25), fpsToY(0), 0.02, color.r, color.g, color.b
      ];
      gl.bufferSubData(gl.ARRAY_BUFFER, 0, new Float32Array(updateVerts));
    } else {
      updateVerts.push(
        segmentToX(this.lastSegment+1), fpsToY(maxFPS), 0.02, color.r, color.g, color.b,
        segmentToX(this.lastSegment+1.25), fpsToY(maxFPS), 0.02, color.r, color.g, color.b,
        segmentToX(this.lastSegment+1), fpsToY(0), 0.02, color.r, color.g, color.b,
        segmentToX(this.lastSegment+1.25), fpsToY(0), 0.02, color.r, color.g, color.b
      );
      gl.bufferSubData(gl.ARRAY_BUFFER, this.lastSegment * 24 * 4, new Float32Array(updateVerts));
    }

    this.lastSegment = (this.lastSegment+1) % segments;
  };

  Stats.prototype.render = function(projectionMat, modelViewMat) {
    var gl = this.gl;

    // Render text first, minor win for early fragment discard
    mat4_multiply(this.modelViewMatrix, modelViewMat, this.textMatrix);
    this.sevenSegmentText.render(projectionMat, this.modelViewMatrix, this.fps + " FP5");

    gl.useProgram(this.program);

    gl.uniformMatrix4fv(this.uniforms.projectionMat, false, projectionMat);
    gl.uniformMatrix4fv(this.uniforms.modelViewMat, false, modelViewMat);

    gl.enableVertexAttribArray(this.attribs.position);
    gl.enableVertexAttribArray(this.attribs.color);

    gl.bindBuffer(gl.ARRAY_BUFFER, this.fpsVertBuffer);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.fpsIndexBuffer);

    gl.vertexAttribPointer(this.attribs.position, 3, gl.FLOAT, false, 24, 0);
    gl.vertexAttribPointer(this.attribs.color, 3, gl.FLOAT, false, 24, 12);

    // Draw the graph and background in a single call
    gl.drawElements(gl.TRIANGLES, this.fpsIndexCount, gl.UNSIGNED_SHORT, 0);
  }

  Stats.prototype.renderOrtho = function(x, y, width, height) {
    var canvas = this.gl.canvas;

    if (x == undefined || y == undefined) {
      x = 10 * window.devicePixelRatio;
      y = 10 * window.devicePixelRatio;
    }
    if (width == undefined || height == undefined) {
      width = 75 * window.devicePixelRatio;
      height = 75 * window.devicePixelRatio;
    }

    mat4_ortho(this.orthoProjMatrix, 0, canvas.width, 0, canvas.height, 0.1, 1024);

    mat4_identity(this.orthoViewMatrix);
    mat4_translate(this.orthoViewMatrix, this.orthoViewMatrix, [x, canvas.height - height - y, -1]);
    mat4_scale(this.orthoViewMatrix, this.orthoViewMatrix, [width, height, 1]);
    mat4_translate(this.orthoViewMatrix, this.orthoViewMatrix, [0.5, 0.5, 0]);

    this.render(this.orthoProjMatrix, this.orthoViewMatrix);
  }

  return Stats;
})();
