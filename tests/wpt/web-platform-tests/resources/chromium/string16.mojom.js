// Copyright 2014 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

'use strict';

(function() {
  var mojomId = 'mojo/public/mojom/base/string16.mojom';
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
  var big_buffer$ =
      mojo.internal.exposeNamespace('mojoBase.mojom');
  if (mojo.config.autoLoadMojomDeps) {
    mojo.internal.loadMojomIfNecessary(
        'mojo/public/mojom/base/big_buffer.mojom', 'big_buffer.mojom.js');
  }



  function String16(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  String16.prototype.initDefaults_ = function() {
    this.data = null;
  };
  String16.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  String16.validate = function(messageValidator, offset) {
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


    // validate String16.data
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 0, 2, codec.Uint16, false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  String16.encodedSize = codec.kStructHeaderSize + 8;

  String16.decode = function(decoder) {
    var packed;
    var val = new String16();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.data = decoder.decodeArrayPointer(codec.Uint16);
    return val;
  };

  String16.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(String16.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeArrayPointer(codec.Uint16, val.data);
  };
  function BigString16(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  BigString16.prototype.initDefaults_ = function() {
    this.data = null;
  };
  BigString16.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  BigString16.validate = function(messageValidator, offset) {
    var err;
    err = messageValidator.validateStructHeader(offset, codec.kStructHeaderSize);
    if (err !== validator.validationError.NONE)
        return err;

    var kVersionSizes = [
      {version: 0, numBytes: 24}
    ];
    err = messageValidator.validateStructVersion(offset, kVersionSizes);
    if (err !== validator.validationError.NONE)
        return err;


    // validate BigString16.data
    err = messageValidator.validateUnion(offset + codec.kStructHeaderSize + 0, big_buffer$.BigBuffer, false);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  BigString16.encodedSize = codec.kStructHeaderSize + 16;

  BigString16.decode = function(decoder) {
    var packed;
    var val = new BigString16();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.data = decoder.decodeStruct(big_buffer$.BigBuffer);
    return val;
  };

  BigString16.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(BigString16.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(big_buffer$.BigBuffer, val.data);
  };
  exports.String16 = String16;
  exports.BigString16 = BigString16;
})();