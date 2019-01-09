// Copyright 2014 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

'use strict';

(function() {
  var mojomId = 'device/usb/public/mojom/device_enumeration_options.mojom';
  if (mojo.internal.isMojomLoaded(mojomId)) {
    console.warn('The following mojom is loaded multiple times: ' + mojomId);
    return;
  }
  mojo.internal.markMojomLoaded(mojomId);
  var bindings = mojo;
  var associatedBindings = mojo;
  var codec = mojo.internal;
  var validator = mojo.internal;

  var exports = mojo.internal.exposeNamespace('device.mojom');
  var string16$ =
      mojo.internal.exposeNamespace('mojoBase.mojom');
  if (mojo.config.autoLoadMojomDeps) {
    mojo.internal.loadMojomIfNecessary(
        'mojo/public/mojom/base/string16.mojom', '../../../../mojo/public/mojom/base/string16.mojom.js');
  }



  function UsbDeviceFilter(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  UsbDeviceFilter.prototype.initDefaults_ = function() {
    this.hasVendorId = false;
    this.hasProductId = false;
    this.hasClassCode = false;
    this.hasSubclassCode = false;
    this.hasProtocolCode = false;
    this.classCode = 0;
    this.vendorId = 0;
    this.productId = 0;
    this.subclassCode = 0;
    this.protocolCode = 0;
    this.serialNumber = null;
  };
  UsbDeviceFilter.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  UsbDeviceFilter.validate = function(messageValidator, offset) {
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












    // validate UsbDeviceFilter.serialNumber
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 8, string16$.String16, true);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  UsbDeviceFilter.encodedSize = codec.kStructHeaderSize + 16;

  UsbDeviceFilter.decode = function(decoder) {
    var packed;
    var val = new UsbDeviceFilter();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    packed = decoder.readUint8();
    val.hasVendorId = (packed >> 0) & 1 ? true : false;
    val.hasProductId = (packed >> 1) & 1 ? true : false;
    val.hasClassCode = (packed >> 2) & 1 ? true : false;
    val.hasSubclassCode = (packed >> 3) & 1 ? true : false;
    val.hasProtocolCode = (packed >> 4) & 1 ? true : false;
    val.classCode = decoder.decodeStruct(codec.Uint8);
    val.vendorId = decoder.decodeStruct(codec.Uint16);
    val.productId = decoder.decodeStruct(codec.Uint16);
    val.subclassCode = decoder.decodeStruct(codec.Uint8);
    val.protocolCode = decoder.decodeStruct(codec.Uint8);
    val.serialNumber = decoder.decodeStructPointer(string16$.String16);
    return val;
  };

  UsbDeviceFilter.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(UsbDeviceFilter.encodedSize);
    encoder.writeUint32(0);
    packed = 0;
    packed |= (val.hasVendorId & 1) << 0
    packed |= (val.hasProductId & 1) << 1
    packed |= (val.hasClassCode & 1) << 2
    packed |= (val.hasSubclassCode & 1) << 3
    packed |= (val.hasProtocolCode & 1) << 4
    encoder.writeUint8(packed);
    encoder.encodeStruct(codec.Uint8, val.classCode);
    encoder.encodeStruct(codec.Uint16, val.vendorId);
    encoder.encodeStruct(codec.Uint16, val.productId);
    encoder.encodeStruct(codec.Uint8, val.subclassCode);
    encoder.encodeStruct(codec.Uint8, val.protocolCode);
    encoder.encodeStructPointer(string16$.String16, val.serialNumber);
  };
  function UsbEnumerationOptions(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  UsbEnumerationOptions.prototype.initDefaults_ = function() {
    this.filters = null;
  };
  UsbEnumerationOptions.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  UsbEnumerationOptions.validate = function(messageValidator, offset) {
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


    // validate UsbEnumerationOptions.filters
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 0, 8, new codec.PointerTo(UsbDeviceFilter), false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  UsbEnumerationOptions.encodedSize = codec.kStructHeaderSize + 8;

  UsbEnumerationOptions.decode = function(decoder) {
    var packed;
    var val = new UsbEnumerationOptions();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.filters = decoder.decodeArrayPointer(new codec.PointerTo(UsbDeviceFilter));
    return val;
  };

  UsbEnumerationOptions.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(UsbEnumerationOptions.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeArrayPointer(new codec.PointerTo(UsbDeviceFilter), val.filters);
  };
  exports.UsbDeviceFilter = UsbDeviceFilter;
  exports.UsbEnumerationOptions = UsbEnumerationOptions;
})();