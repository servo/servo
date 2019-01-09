// Copyright 2014 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

'use strict';

(function() {
  var mojomId = 'mojo/public/mojom/base/big_buffer.mojom';
  if (mojo.internal.isMojomLoaded(mojomId)) {
    console.warn('The following mojom is loaded multiple times: ' + mojomId);
    return;
  }
  mojo.internal.markMojomLoaded(mojomId);
  var bindings = mojo;
  var associatedBindings = mojo;
  var codec = mojo.internal;
  var validator = mojo.internal;

  var exports = mojo.internal.exposeNamespace('mojoBase.mojom');



  function BigBufferSharedMemoryRegion(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  BigBufferSharedMemoryRegion.prototype.initDefaults_ = function() {
    this.bufferHandle = null;
    this.size = 0;
  };
  BigBufferSharedMemoryRegion.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  BigBufferSharedMemoryRegion.validate = function(messageValidator, offset) {
    var err;
    err = messageValidator.validateStructHeader(offset, codec.kStructHeaderSize);
    if (err !== validator.validationError.NONE)
        return err;

    var kVersionSizes = [
      {version: 0, numBytes: 16}
    ];
    err = messageValidator.validateStructVersion(offset, kVersionSizes);
    if (err !== validator.validationError.NONE)
        return err;


    // validate BigBufferSharedMemoryRegion.bufferHandle
    err = messageValidator.validateHandle(offset + codec.kStructHeaderSize + 0, false)
    if (err !== validator.validationError.NONE)
        return err;


    return validator.validationError.NONE;
  };

  BigBufferSharedMemoryRegion.encodedSize = codec.kStructHeaderSize + 8;

  BigBufferSharedMemoryRegion.decode = function(decoder) {
    var packed;
    var val = new BigBufferSharedMemoryRegion();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.bufferHandle = decoder.decodeStruct(codec.Handle);
    val.size = decoder.decodeStruct(codec.Uint32);
    return val;
  };

  BigBufferSharedMemoryRegion.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(BigBufferSharedMemoryRegion.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.Handle, val.bufferHandle);
    encoder.encodeStruct(codec.Uint32, val.size);
  };

  function BigBuffer(value) {
    this.initDefault_();
    this.initValue_(value);
  }


  BigBuffer.Tags = {
    bytes: 0,
    sharedMemory: 1,
    invalidBuffer: 2,
  };

  BigBuffer.prototype.initDefault_ = function() {
    this.$data = null;
    this.$tag = undefined;
  }

  BigBuffer.prototype.initValue_ = function(value) {
    if (value == undefined) {
      return;
    }

    var keys = Object.keys(value);
    if (keys.length == 0) {
      return;
    }

    if (keys.length > 1) {
      throw new TypeError("You may set only one member on a union.");
    }

    var fields = [
        "bytes",
        "sharedMemory",
        "invalidBuffer",
    ];

    if (fields.indexOf(keys[0]) < 0) {
      throw new ReferenceError(keys[0] + " is not a BigBuffer member.");

    }

    this[keys[0]] = value[keys[0]];
  }
  Object.defineProperty(BigBuffer.prototype, "bytes", {
    get: function() {
      if (this.$tag != BigBuffer.Tags.bytes) {
        throw new ReferenceError(
            "BigBuffer.bytes is not currently set.");
      }
      return this.$data;
    },

    set: function(value) {
      this.$tag = BigBuffer.Tags.bytes;
      this.$data = value;
    }
  });
  Object.defineProperty(BigBuffer.prototype, "sharedMemory", {
    get: function() {
      if (this.$tag != BigBuffer.Tags.sharedMemory) {
        throw new ReferenceError(
            "BigBuffer.sharedMemory is not currently set.");
      }
      return this.$data;
    },

    set: function(value) {
      this.$tag = BigBuffer.Tags.sharedMemory;
      this.$data = value;
    }
  });
  Object.defineProperty(BigBuffer.prototype, "invalidBuffer", {
    get: function() {
      if (this.$tag != BigBuffer.Tags.invalidBuffer) {
        throw new ReferenceError(
            "BigBuffer.invalidBuffer is not currently set.");
      }
      return this.$data;
    },

    set: function(value) {
      this.$tag = BigBuffer.Tags.invalidBuffer;
      this.$data = value;
    }
  });


    BigBuffer.encode = function(encoder, val) {
      if (val == null) {
        encoder.writeUint64(0);
        encoder.writeUint64(0);
        return;
      }
      if (val.$tag == undefined) {
        throw new TypeError("Cannot encode unions with an unknown member set.");
      }

      encoder.writeUint32(16);
      encoder.writeUint32(val.$tag);
      switch (val.$tag) {
        case BigBuffer.Tags.bytes:
          encoder.encodeArrayPointer(codec.Uint8, val.bytes);
          break;
        case BigBuffer.Tags.sharedMemory:
          encoder.encodeStructPointer(BigBufferSharedMemoryRegion, val.sharedMemory);
          break;
        case BigBuffer.Tags.invalidBuffer:
          encoder.writeUint8(val.invalidBuffer ? 1 : 0);
          break;
      }
      encoder.align();
    };


    BigBuffer.decode = function(decoder) {
      var size = decoder.readUint32();
      if (size == 0) {
        decoder.readUint32();
        decoder.readUint64();
        return null;
      }

      var result = new BigBuffer();
      var tag = decoder.readUint32();
      switch (tag) {
        case BigBuffer.Tags.bytes:
          result.bytes = decoder.decodeArrayPointer(codec.Uint8);
          break;
        case BigBuffer.Tags.sharedMemory:
          result.sharedMemory = decoder.decodeStructPointer(BigBufferSharedMemoryRegion);
          break;
        case BigBuffer.Tags.invalidBuffer:
          result.invalidBuffer = decoder.readUint8() ? true : false;
          break;
      }
      decoder.align();

      return result;
    };


    BigBuffer.validate = function(messageValidator, offset) {
      var size = messageValidator.decodeUnionSize(offset);
      if (size != 16) {
        return validator.validationError.INVALID_UNION_SIZE;
      }

      var tag = messageValidator.decodeUnionTag(offset);
      var data_offset = offset + 8;
      var err;
      switch (tag) {
        case BigBuffer.Tags.bytes:


    // validate BigBuffer.bytes
    err = messageValidator.validateArrayPointer(data_offset, 1, codec.Uint8, false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;
          break;
        case BigBuffer.Tags.sharedMemory:


    // validate BigBuffer.sharedMemory
    err = messageValidator.validateStructPointer(data_offset, BigBufferSharedMemoryRegion, false);
    if (err !== validator.validationError.NONE)
        return err;
          break;
        case BigBuffer.Tags.invalidBuffer:


          break;
      }

      return validator.validationError.NONE;
    };

  BigBuffer.encodedSize = 16;
  exports.BigBufferSharedMemoryRegion = BigBufferSharedMemoryRegion;
  exports.BigBuffer = BigBuffer;
})();
