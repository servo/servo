/*
Copyright (c) 2015, Brandon Jones.

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
Utility class to make loading shader programs easier. Does all the error
checking you typically want, automatically queries uniform and attribute
locations, and attempts to take advantage of some browser's ability to link
asynchronously by not querying any information from the program until it's
first use.
*/
var WGLUProgram = (function() {

  "use strict";

  // Attempts to allow the browser to asynchronously compile and link
  var Program = function(gl) {
    this.gl = gl;
    this.program = gl.createProgram();
    this.attrib = null;
    this.uniform = null;

    this._firstUse = true;
    this._vertexShader = null;
    this._fragmentShader = null;
  }

  Program.prototype.attachShaderSource = function(source, type) {
    var gl = this.gl;
    var shader;

    switch (type) {
      case gl.VERTEX_SHADER:
        this._vertexShader = gl.createShader(type);
        shader = this._vertexShader;
        break;
      case gl.FRAGMENT_SHADER:
        this._fragmentShader = gl.createShader(type);
        shader = this._fragmentShader;
        break;
      default:
        console.Error("Invalid Shader Type:", type);
        return;
    }

    gl.attachShader(this.program, shader);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
  }

  Program.prototype.attachShaderSourceFromXHR = function(url, type) {
    var self = this;
    return new Promise(function(resolve, reject) {
      var xhr = new XMLHttpRequest();
      xhr.addEventListener("load", function (ev) {
        if (xhr.status == 200) {
          self.attachShaderSource(xhr.response, type);
          resolve();
        } else {
          reject(xhr.statusText);
        }
      }, false);
      xhr.open("GET", url, true);
      xhr.send(null);
    });
  }

  Program.prototype.attachShaderSourceFromTag = function(tagId, type) {
    var shaderTag = document.getElementById(tagId);
    if (!shaderTag) {
      console.error("Shader source tag not found:", tagId);
      return;
    }

    if (!type) {
      if (shaderTag.type == "x-shader/x-vertex") {
        type = this.gl.VERTEX_SHADER;
      } else if (shaderTag.type == "x-shader/x-fragment") {
        type = this.gl.FRAGMENT_SHADER;
      } else {
        console.error("Invalid Shader Type:", shaderTag.type);
        return;
      }
    }

    var src = "";
    var k = shaderTag.firstChild;
    while (k) {
      if (k.nodeType == 3) {
        src += k.textContent;
      }
      k = k.nextSibling;
    }
    this.attachShaderSource(src, type);
  }

  Program.prototype.bindAttribLocation = function(attribLocationMap) {
    var gl = this.gl;

    if (attribLocationMap) {
      this.attrib = {};
      for (var attribName in attribLocationMap) {
        gl.bindAttribLocation(this.program, attribLocationMap[attribName], attribName);
        this.attrib[attribName] = attribLocationMap[attribName];
      }
    }
  }

  Program.prototype.transformFeedbackVaryings = function(varyings, type) {
    gl.transformFeedbackVaryings(this.program, varyings, type);
  }

  Program.prototype.link = function() {
    this.gl.linkProgram(this.program);
  }

  Program.prototype.use = function() {
    var gl = this.gl;

    // If this is the first time the program has been used do all the error checking and
    // attrib/uniform querying needed.
    if (this._firstUse) {
      if (!gl.getProgramParameter(this.program, gl.LINK_STATUS)) {
        if (this._vertexShader && !gl.getShaderParameter(this._vertexShader, gl.COMPILE_STATUS)) {
          console.error("Vertex shader compile error:", gl.getShaderInfoLog(this._vertexShader));
        } else if (this._fragmentShader && !gl.getShaderParameter(this._fragmentShader, gl.COMPILE_STATUS)) {
          console.error("Fragment shader compile error:", gl.getShaderInfoLog(this._fragmentShader));
        } else {
          console.error("Program link error:", gl.getProgramInfoLog(this.program));
        }
        gl.deleteProgram(this.program);
        this.program = null;
      } else {
        if (!this.attrib) {
          this.attrib = {};
          var attribCount = gl.getProgramParameter(this.program, gl.ACTIVE_ATTRIBUTES);
          for (var i = 0; i < attribCount; i++) {
            var attribInfo = gl.getActiveAttrib(this.program, i);
            this.attrib[attribInfo.name] = gl.getAttribLocation(this.program, attribInfo.name);
          }
        }

        this.uniform = {};
        var uniformCount = gl.getProgramParameter(this.program, gl.ACTIVE_UNIFORMS);
        var uniformName = "";
        for (var i = 0; i < uniformCount; i++) {
          var uniformInfo = gl.getActiveUniform(this.program, i);
          uniformName = uniformInfo.name.replace("[0]", "");
          this.uniform[uniformName] = gl.getUniformLocation(this.program, uniformName);
        }
      }
      gl.deleteShader(this._vertexShader);
      gl.deleteShader(this._fragmentShader);
      this._firstUse = false;
    }

    gl.useProgram(this.program);
  }

  return Program;
})();
