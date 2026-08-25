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
Handles loading of textures of mutliple formats, tries to be efficent about it.

Formats supported will vary by devices. Use the .supports<format>() functions
to determine if a format is supported. Most of the time you can just call
loader.loadTexture("url"); and it will handle it based on the extension.
If the extension can't be relied on use the corresponding
.load<Extension>("url") calls.
*/
var WGLUTextureLoader = (function() {

  "use strict";

  //============================//
  // DXT constants and utilites //
  //============================//

  // Utility functions
  // Builds a numeric code for a given fourCC string
  function fourCCToInt32(value) {
    return value.charCodeAt(0) +
      (value.charCodeAt(1) << 8) +
      (value.charCodeAt(2) << 16) +
      (value.charCodeAt(3) << 24);
  }

  // Turns a fourCC numeric code into a string
  function int32ToFourCC(value) {
    return String.fromCharCode(
      value & 0xff,
      (value >> 8) & 0xff,
      (value >> 16) & 0xff,
      (value >> 24) & 0xff
    );
  }

  // Calcualates the size of a compressed texture level in bytes
  function textureLevelSize(format, width, height) {
    switch (format) {
      case COMPRESSED_RGB_S3TC_DXT1_EXT:
      case COMPRESSED_RGB_ATC_WEBGL:
      case COMPRESSED_RGB_ETC1_WEBGL:
        return ((width + 3) >> 2) * ((height + 3) >> 2) * 8;

      case COMPRESSED_RGBA_S3TC_DXT3_EXT:
      case COMPRESSED_RGBA_S3TC_DXT5_EXT:
      case COMPRESSED_RGBA_ATC_EXPLICIT_ALPHA_WEBGL:
      case COMPRESSED_RGBA_ATC_INTERPOLATED_ALPHA_WEBGL:
        return ((width + 3) >> 2) * ((height + 3) >> 2) * 16;

      case COMPRESSED_RGB_PVRTC_4BPPV1_IMG:
      case COMPRESSED_RGBA_PVRTC_4BPPV1_IMG:
        return Math.floor((Math.max(width, 8) * Math.max(height, 8) * 4 + 7) / 8);

      case COMPRESSED_RGB_PVRTC_2BPPV1_IMG:
      case COMPRESSED_RGBA_PVRTC_2BPPV1_IMG:
        return Math.floor((Math.max(width, 16) * Math.max(height, 8) * 2 + 7) / 8);

      default:
        return 0;
    }
  }

  // DXT formats, from:
  // http://www.khronos.org/registry/webgl/extensions/WEBGL_compressed_texture_s3tc/
  var COMPRESSED_RGB_S3TC_DXT1_EXT  = 0x83F0;
  var COMPRESSED_RGBA_S3TC_DXT1_EXT = 0x83F1;
  var COMPRESSED_RGBA_S3TC_DXT3_EXT = 0x83F2;
  var COMPRESSED_RGBA_S3TC_DXT5_EXT = 0x83F3;

  // ATC formats, from:
  // http://www.khronos.org/registry/webgl/extensions/WEBGL_compressed_texture_atc/
  var COMPRESSED_RGB_ATC_WEBGL                     = 0x8C92;
  var COMPRESSED_RGBA_ATC_EXPLICIT_ALPHA_WEBGL     = 0x8C93;
  var COMPRESSED_RGBA_ATC_INTERPOLATED_ALPHA_WEBGL = 0x87EE;

  // DXT values and structures referenced from:
  // http://msdn.microsoft.com/en-us/library/bb943991.aspx/
  var DDS_MAGIC = 0x20534444;
  var DDSD_MIPMAPCOUNT = 0x20000;
  var DDPF_FOURCC = 0x4;

  var DDS_HEADER_LENGTH = 31; // The header length in 32 bit ints.

  // Offsets into the header array.
  var DDS_HEADER_MAGIC = 0;

  var DDS_HEADER_SIZE = 1;
  var DDS_HEADER_FLAGS = 2;
  var DDS_HEADER_HEIGHT = 3;
  var DDS_HEADER_WIDTH = 4;

  var DDS_HEADER_MIPMAPCOUNT = 7;

  var DDS_HEADER_PF_FLAGS = 20;
  var DDS_HEADER_PF_FOURCC = 21;

  // FourCC format identifiers.
  var FOURCC_DXT1 = fourCCToInt32("DXT1");
  var FOURCC_DXT3 = fourCCToInt32("DXT3");
  var FOURCC_DXT5 = fourCCToInt32("DXT5");

  var FOURCC_ATC = fourCCToInt32("ATC ");
  var FOURCC_ATCA = fourCCToInt32("ATCA");
  var FOURCC_ATCI = fourCCToInt32("ATCI");

  //==================//
  // Crunch constants //
  //==================//

  // Taken from crnlib.h
  var CRN_FORMAT = {
    cCRNFmtInvalid: -1,

    cCRNFmtDXT1: 0,
    // cCRNFmtDXT3 is not currently supported when writing to CRN - only DDS.
    cCRNFmtDXT3: 1,
    cCRNFmtDXT5: 2

    // Crunch supports more formats than this, but we can't use them here.
  };

  // Mapping of Crunch formats to DXT formats.
  var DXT_FORMAT_MAP = {};
  DXT_FORMAT_MAP[CRN_FORMAT.cCRNFmtDXT1] = COMPRESSED_RGB_S3TC_DXT1_EXT;
  DXT_FORMAT_MAP[CRN_FORMAT.cCRNFmtDXT3] = COMPRESSED_RGBA_S3TC_DXT3_EXT;
  DXT_FORMAT_MAP[CRN_FORMAT.cCRNFmtDXT5] = COMPRESSED_RGBA_S3TC_DXT5_EXT;

  //===============//
  // PVR constants //
  //===============//

  // PVR formats, from:
  // http://www.khronos.org/registry/webgl/extensions/WEBGL_compressed_texture_pvrtc/
  var COMPRESSED_RGB_PVRTC_4BPPV1_IMG  = 0x8C00;
  var COMPRESSED_RGB_PVRTC_2BPPV1_IMG  = 0x8C01;
  var COMPRESSED_RGBA_PVRTC_4BPPV1_IMG = 0x8C02;
  var COMPRESSED_RGBA_PVRTC_2BPPV1_IMG = 0x8C03;

  // ETC1 format, from:
  // http://www.khronos.org/registry/webgl/extensions/WEBGL_compressed_texture_etc1/
  var COMPRESSED_RGB_ETC1_WEBGL = 0x8D64;

  var PVR_FORMAT_2BPP_RGB  = 0;
  var PVR_FORMAT_2BPP_RGBA = 1;
  var PVR_FORMAT_4BPP_RGB  = 2;
  var PVR_FORMAT_4BPP_RGBA = 3;
  var PVR_FORMAT_ETC1      = 6;
  var PVR_FORMAT_DXT1      = 7;
  var PVR_FORMAT_DXT3      = 9;
  var PVR_FORMAT_DXT5      = 5;

  var PVR_HEADER_LENGTH = 13; // The header length in 32 bit ints.
  var PVR_MAGIC = 0x03525650; //0x50565203;

  // Offsets into the header array.
  var PVR_HEADER_MAGIC = 0;
  var PVR_HEADER_FORMAT = 2;
  var PVR_HEADER_HEIGHT = 6;
  var PVR_HEADER_WIDTH = 7;
  var PVR_HEADER_MIPMAPCOUNT = 11;
  var PVR_HEADER_METADATA = 12;

  //============//
  // Misc Utils //
  //============//

  // When an error occurs set the texture to a 1x1 black pixel
  // This prevents WebGL errors from attempting to use unrenderable textures
  // and clears out stale data if we're re-using a texture.
  function clearOnError(gl, error, texture, callback) {
    if (console) {
      console.error(error);
    }
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, new Uint8Array([0, 0, 0]));
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);

    // Notify the user that an error occurred and the texture is ready.
    if (callback) { callback(texture, error, null); }
  }

  function isPowerOfTwo(n) {
    return (n & (n - 1)) === 0;
  }

  function getExtension(gl, name) {
    var vendorPrefixes = ["", "WEBKIT_", "MOZ_"];
    var ext = null;
    for (var i in vendorPrefixes) {
      ext = gl.getExtension(vendorPrefixes[i] + name);
      if (ext) { break; }
    }
    return ext;
  }

  //==================//
  // DDS File Reading //
  //==================//

  // Parse a DDS file and provide information about the raw DXT data it contains to the given callback.
  function parseDDS(arrayBuffer, callback, errorCallback) {
    // Callbacks must be provided.
    if (!callback || !errorCallback) { return; }

    // Get a view of the arrayBuffer that represents the DDS header.
    var header = new Int32Array(arrayBuffer, 0, DDS_HEADER_LENGTH);

    // Do some sanity checks to make sure this is a valid DDS file.
    if(header[DDS_HEADER_MAGIC] != DDS_MAGIC) {
      errorCallback("Invalid magic number in DDS header");
      return 0;
    }

    if(!header[DDS_HEADER_PF_FLAGS] & DDPF_FOURCC) {
      errorCallback("Unsupported format, must contain a FourCC code");
      return 0;
    }

    // Determine what type of compressed data the file contains.
    var fourCC = header[DDS_HEADER_PF_FOURCC];
    var internalFormat;
    switch(fourCC) {
      case FOURCC_DXT1:
        internalFormat = COMPRESSED_RGB_S3TC_DXT1_EXT;
        break;

      case FOURCC_DXT3:
        internalFormat = COMPRESSED_RGBA_S3TC_DXT3_EXT;
        break;

      case FOURCC_DXT5:
        internalFormat = COMPRESSED_RGBA_S3TC_DXT5_EXT;
        break;

      case FOURCC_ATC:
        internalFormat = COMPRESSED_RGB_ATC_WEBGL;
        break;

      case FOURCC_ATCA:
        internalFormat = COMPRESSED_RGBA_ATC_EXPLICIT_ALPHA_WEBGL;
        break;

      case FOURCC_ATCI:
        internalFormat = COMPRESSED_RGBA_ATC_INTERPOLATED_ALPHA_WEBGL;
        break;


      default:
        errorCallback("Unsupported FourCC code: " + int32ToFourCC(fourCC));
        return;
    }

    // Determine how many mipmap levels the file contains.
    var levels = 1;
    if(header[DDS_HEADER_FLAGS] & DDSD_MIPMAPCOUNT) {
      levels = Math.max(1, header[DDS_HEADER_MIPMAPCOUNT]);
    }

    // Gather other basic metrics and a view of the raw the DXT data.
    var width = header[DDS_HEADER_WIDTH];
    var height = header[DDS_HEADER_HEIGHT];
    var dataOffset = header[DDS_HEADER_SIZE] + 4;
    var dxtData = new Uint8Array(arrayBuffer, dataOffset);

    // Pass the DXT information to the callback for uploading.
    callback(dxtData, width, height, levels, internalFormat);
  }

  //==================//
  // PVR File Reading //
  //==================//

  // Parse a PVR file and provide information about the raw texture data it contains to the given callback.
  function parsePVR(arrayBuffer, callback, errorCallback) {
    // Callbacks must be provided.
    if (!callback || !errorCallback) { return; }

    // Get a view of the arrayBuffer that represents the DDS header.
    var header = new Int32Array(arrayBuffer, 0, PVR_HEADER_LENGTH);

    // Do some sanity checks to make sure this is a valid DDS file.
    if(header[PVR_HEADER_MAGIC] != PVR_MAGIC) {
      errorCallback("Invalid magic number in PVR header");
      return 0;
    }

    // Determine what type of compressed data the file contains.
    var format = header[PVR_HEADER_FORMAT];
    var internalFormat;
    switch(format) {
      case PVR_FORMAT_2BPP_RGB:
        internalFormat = COMPRESSED_RGB_PVRTC_2BPPV1_IMG;
        break;

      case PVR_FORMAT_2BPP_RGBA:
        internalFormat = COMPRESSED_RGBA_PVRTC_2BPPV1_IMG;
        break;

      case PVR_FORMAT_4BPP_RGB:
        internalFormat = COMPRESSED_RGB_PVRTC_4BPPV1_IMG;
        break;

      case PVR_FORMAT_4BPP_RGBA:
        internalFormat = COMPRESSED_RGBA_PVRTC_4BPPV1_IMG;
        break;

      case PVR_FORMAT_ETC1:
        internalFormat = COMPRESSED_RGB_ETC1_WEBGL;
        break;

      case PVR_FORMAT_DXT1:
        internalFormat = COMPRESSED_RGB_S3TC_DXT1_EXT;
        break;

      case PVR_FORMAT_DXT3:
        internalFormat = COMPRESSED_RGBA_S3TC_DXT3_EXT;
        break;

      case PVR_FORMAT_DXT5:
        internalFormat = COMPRESSED_RGBA_S3TC_DXT5_EXT;
        break;

      default:
        errorCallback("Unsupported PVR format: " + format);
        return;
    }

    // Gather other basic metrics and a view of the raw the DXT data.
    var width = header[PVR_HEADER_WIDTH];
    var height = header[PVR_HEADER_HEIGHT];
    var levels = header[PVR_HEADER_MIPMAPCOUNT];
    var dataOffset = header[PVR_HEADER_METADATA] + 52;
    var pvrtcData = new Uint8Array(arrayBuffer, dataOffset);

    // Pass the PVRTC information to the callback for uploading.
    callback(pvrtcData, width, height, levels, internalFormat);
  }

  //=============//
  // IMG loading //
  //=============//

  /*
  This function provides a method for loading webgl textures using a pool of
  image elements, which has very low memory overhead. For more details see:
  http://blog.tojicode.com/2012/03/javascript-memory-optimization-and.html
  */
  var loadImgTexture = (function createTextureLoader() {
    var MAX_CACHE_IMAGES = 16;

    var textureImageCache = new Array(MAX_CACHE_IMAGES);
    var cacheTop = 0;
    var remainingCacheImages = MAX_CACHE_IMAGES;
    var pendingTextureRequests = [];

    var TextureImageLoader = function(loadedCallback) {
      var self = this;
      var blackPixel = new Uint8Array([0, 0, 0]);

      this.gl = null;
      this.texture = null;
      this.callback = null;

      this.image = new Image();
      this.image.crossOrigin = 'anonymous';
      this.image.addEventListener('load', function() {
        var gl = self.gl;
        gl.bindTexture(gl.TEXTURE_2D, self.texture);

        var startTime = Date.now();
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, self.image);

        if (isPowerOfTwo(self.image.width) && isPowerOfTwo(self.image.height)) {
          gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
          gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_NEAREST);
          gl.generateMipmap(gl.TEXTURE_2D);
        } else {
          gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
          gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        }
        var uploadTime = Date.now() - startTime;

        if(self.callback) {
          var stats = {
            width: self.image.width,
            height: self.image.height,
            internalFormat: gl.RGBA,
            levelZeroSize: self.image.width * self.image.height * 4,
            uploadTime: uploadTime
          };
          self.callback(self.texture, null, stats);
        }
        loadedCallback(self);
      }, false);
      this.image.addEventListener('error', function(ev) {
        clearOnError(self.gl, 'Image could not be loaded: ' + self.image.src, self.texture, self.callback);
        loadedCallback(self);
      }, false);
    };

    TextureImageLoader.prototype.loadTexture = function(gl, src, texture, callback) {
      this.gl = gl;
      this.texture = texture;
      this.callback = callback;
      this.image.src = src;
    };

    var PendingTextureRequest = function(gl, src, texture, callback) {
      this.gl = gl;
      this.src = src;
      this.texture = texture;
      this.callback = callback;
    };

    function releaseTextureImageLoader(til) {
      var req;
      if(pendingTextureRequests.length) {
        req = pendingTextureRequests.shift();
        til.loadTexture(req.gl, req.src, req.texture, req.callback);
      } else {
        textureImageCache[cacheTop++] = til;
      }
    }

    return function(gl, src, texture, callback) {
      var til;

      if(cacheTop) {
        til = textureImageCache[--cacheTop];
        til.loadTexture(gl, src, texture, callback);
      } else if (remainingCacheImages) {
        til = new TextureImageLoader(releaseTextureImageLoader);
        til.loadTexture(gl, src, texture, callback);
        --remainingCacheImages;
      } else {
        pendingTextureRequests.push(new PendingTextureRequest(gl, src, texture, callback));
      }

      return texture;
    };
  })();

  //=====================//
  // TextureLoader Class //
  //=====================//

  // This class is our public interface.
  var TextureLoader = function(gl) {
    this.gl = gl;

    // Load the compression format extensions, if available
    this.dxtExt = getExtension(gl, "WEBGL_compressed_texture_s3tc");
    this.pvrtcExt = getExtension(gl, "WEBGL_compressed_texture_pvrtc");
    this.atcExt = getExtension(gl, "WEBGL_compressed_texture_atc");
    this.etc1Ext = getExtension(gl, "WEBGL_compressed_texture_etc1");

    // Returns whether or not the compressed format is supported by the WebGL implementation
    TextureLoader.prototype._formatSupported = function(format) {
      switch (format) {
        case COMPRESSED_RGB_S3TC_DXT1_EXT:
        case COMPRESSED_RGBA_S3TC_DXT3_EXT:
        case COMPRESSED_RGBA_S3TC_DXT5_EXT:
          return !!this.dxtExt;

        case COMPRESSED_RGB_PVRTC_4BPPV1_IMG:
        case COMPRESSED_RGBA_PVRTC_4BPPV1_IMG:
        case COMPRESSED_RGB_PVRTC_2BPPV1_IMG:
        case COMPRESSED_RGBA_PVRTC_2BPPV1_IMG:
          return !!this.pvrtcExt;

        case COMPRESSED_RGB_ATC_WEBGL:
        case COMPRESSED_RGBA_ATC_EXPLICIT_ALPHA_WEBGL:
        case COMPRESSED_RGBA_ATC_INTERPOLATED_ALPHA_WEBGL:
          return !!this.atcExt;

        case COMPRESSED_RGB_ETC1_WEBGL:
          return !!this.etc1Ext;

        default:
          return false;
      }
    }

    // Uploads compressed texture data to the GPU.
    TextureLoader.prototype._uploadCompressedData = function(data, width, height, levels, internalFormat, texture, callback) {
      var gl = this.gl;
      gl.bindTexture(gl.TEXTURE_2D, texture);

      var offset = 0;

      var stats = {
        width: width,
        height: height,
        internalFormat: internalFormat,
        levelZeroSize: textureLevelSize(internalFormat, width, height),
        uploadTime: 0
      };

      var startTime = Date.now();
      // Loop through each mip level of compressed texture data provided and upload it to the given texture.
      for (var i = 0; i < levels; ++i) {
        // Determine how big this level of compressed texture data is in bytes.
        var levelSize = textureLevelSize(internalFormat, width, height);
        // Get a view of the bytes for this level of DXT data.
        var dxtLevel = new Uint8Array(data.buffer, data.byteOffset + offset, levelSize);
        // Upload!
        gl.compressedTexImage2D(gl.TEXTURE_2D, i, internalFormat, width, height, 0, dxtLevel);
        // The next mip level will be half the height and width of this one.
        width = width >> 1;
        height = height >> 1;
        // Advance the offset into the compressed texture data past the current mip level's data.
        offset += levelSize;
      }
      stats.uploadTime = Date.now() - startTime;

      // We can't use gl.generateMipmaps with compressed textures, so only use
      // mipmapped filtering if the compressed texture data contained mip levels.
      if (levels > 1) {
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_NEAREST);
      } else {
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      }

      // Notify the user that the texture is ready.
      if (callback) { callback(texture, null, stats); }
    }

    TextureLoader.prototype.supportsDXT = function() {
      return !!this.dxtExt;
    }

    TextureLoader.prototype.supportsPVRTC = function() {
      return !!this.pvrtcExt;
    }

    TextureLoader.prototype.supportsATC = function() {
      return !!this.atcExt;
    }

    TextureLoader.prototype.supportsETC1 = function() {
      return !!this.etc1Ext;
    }

    // Loads a image file into the given texture.
    // Supports any format that can be loaded into an img tag
    // If no texture is provided one is created and returned.
    TextureLoader.prototype.loadIMG = function(src, texture, callback) {
      if(!texture) {
        texture = this.gl.createTexture();
      }

      loadImgTexture(gl, src, texture, callback);

      return texture;
    }

    // Loads a DDS file into the given texture.
    // If no texture is provided one is created and returned.
    TextureLoader.prototype.loadDDS = function(src, texture, callback) {
      var self = this;
      if (!texture) {
        texture = this.gl.createTexture();
      }

      // Load the file via XHR.
      var xhr = new XMLHttpRequest();
      xhr.addEventListener('load', function (ev) {
        if (xhr.status == 200) {
          // If the file loaded successfully parse it.
          parseDDS(xhr.response, function(dxtData, width, height, levels, internalFormat) {
            if (!self._formatSupported(internalFormat)) {
              clearOnError(self.gl, "Texture format not supported", texture, callback);
              return;
            }
            // Upload the parsed DXT data to the texture.
            self._uploadCompressedData(dxtData, width, height, levels, internalFormat, texture, callback);
          }, function(error) {
            clearOnError(self.gl, error, texture, callback);
          });
        } else {
          clearOnError(self.gl, xhr.statusText, texture, callback);
        }
      }, false);
      xhr.open('GET', src, true);
      xhr.responseType = 'arraybuffer';
      xhr.send(null);

      return texture;
    }

    // Loads a PVR file into the given texture.
    // If no texture is provided one is created and returned.
    TextureLoader.prototype.loadPVR = function(src, texture, callback) {
      var self = this;
      if(!texture) {
        texture = this.gl.createTexture();
      }

      // Load the file via XHR.
      var xhr = new XMLHttpRequest();
      xhr.addEventListener('load', function (ev) {
        if (xhr.status == 200) {
          // If the file loaded successfully parse it.
          parsePVR(xhr.response, function(dxtData, width, height, levels, internalFormat) {
            if (!self._formatSupported(internalFormat)) {
              clearOnError(self.gl, "Texture format not supported", texture, callback);
              return;
            }
            // Upload the parsed PVR data to the texture.
            self._uploadCompressedData(dxtData, width, height, levels, internalFormat, texture, callback);
          }, function(error) {
            clearOnError(self.gl, error, texture, callback);
          });
        } else {
          clearOnError(self.gl, xhr.statusText, texture, callback);
        }
      }, false);
      xhr.open('GET', src, true);
      xhr.responseType = 'arraybuffer';
      xhr.send(null);

      return texture;
    }

    // Loads a texture from a file. Guesses the type based on extension.
    // If no texture is provided one is created and returned.
    TextureLoader.prototype.loadTexture = function(src, texture, callback) {
      // Shamelessly lifted from StackOverflow :)
      // http://stackoverflow.com/questions/680929
      var re = /(?:\.([^.]+))?$/;
      var ext = re.exec(src)[1] || '';
      ext = ext.toLowerCase();

      switch(ext) {
        case 'dds':
          return this.loadDDS(src, texture, callback);
        case 'pvr':
          return this.loadPVR(src, texture, callback);
        default:
          return this.loadIMG(src, texture, callback);
      }
    }

    // Sets a texture to a solid RGBA color
    // If no texture is provided one is created and returned.
    TextureLoader.prototype.makeSolidColor = function(r, g, b, a, texture) {
      var gl = this.gl;
      var data = new Uint8Array([r, g, b, a]);
      if(!texture) {
        texture = gl.createTexture();
      }
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, data);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      return texture;
    }
  }

  return TextureLoader;
})();
