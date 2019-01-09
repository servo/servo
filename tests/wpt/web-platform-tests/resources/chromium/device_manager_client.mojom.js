// Copyright 2014 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

'use strict';

(function() {
  var mojomId = 'device/usb/public/mojom/device_manager_client.mojom';
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
  var device$ =
      mojo.internal.exposeNamespace('device.mojom');
  if (mojo.config.autoLoadMojomDeps) {
    mojo.internal.loadMojomIfNecessary(
        'device/usb/public/mojom/device.mojom', 'device.mojom.js');
  }



  function UsbDeviceManagerClient_OnDeviceAdded_Params(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  UsbDeviceManagerClient_OnDeviceAdded_Params.prototype.initDefaults_ = function() {
    this.deviceInfo = null;
  };
  UsbDeviceManagerClient_OnDeviceAdded_Params.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  UsbDeviceManagerClient_OnDeviceAdded_Params.validate = function(messageValidator, offset) {
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


    // validate UsbDeviceManagerClient_OnDeviceAdded_Params.deviceInfo
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 0, device$.UsbDeviceInfo, false);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  UsbDeviceManagerClient_OnDeviceAdded_Params.encodedSize = codec.kStructHeaderSize + 8;

  UsbDeviceManagerClient_OnDeviceAdded_Params.decode = function(decoder) {
    var packed;
    var val = new UsbDeviceManagerClient_OnDeviceAdded_Params();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.deviceInfo = decoder.decodeStructPointer(device$.UsbDeviceInfo);
    return val;
  };

  UsbDeviceManagerClient_OnDeviceAdded_Params.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(UsbDeviceManagerClient_OnDeviceAdded_Params.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStructPointer(device$.UsbDeviceInfo, val.deviceInfo);
  };
  function UsbDeviceManagerClient_OnDeviceRemoved_Params(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  UsbDeviceManagerClient_OnDeviceRemoved_Params.prototype.initDefaults_ = function() {
    this.deviceInfo = null;
  };
  UsbDeviceManagerClient_OnDeviceRemoved_Params.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  UsbDeviceManagerClient_OnDeviceRemoved_Params.validate = function(messageValidator, offset) {
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


    // validate UsbDeviceManagerClient_OnDeviceRemoved_Params.deviceInfo
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 0, device$.UsbDeviceInfo, false);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  UsbDeviceManagerClient_OnDeviceRemoved_Params.encodedSize = codec.kStructHeaderSize + 8;

  UsbDeviceManagerClient_OnDeviceRemoved_Params.decode = function(decoder) {
    var packed;
    var val = new UsbDeviceManagerClient_OnDeviceRemoved_Params();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.deviceInfo = decoder.decodeStructPointer(device$.UsbDeviceInfo);
    return val;
  };

  UsbDeviceManagerClient_OnDeviceRemoved_Params.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(UsbDeviceManagerClient_OnDeviceRemoved_Params.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStructPointer(device$.UsbDeviceInfo, val.deviceInfo);
  };
  var kUsbDeviceManagerClient_OnDeviceAdded_Name = 0;
  var kUsbDeviceManagerClient_OnDeviceRemoved_Name = 1;

  function UsbDeviceManagerClientPtr(handleOrPtrInfo) {
    this.ptr = new bindings.InterfacePtrController(UsbDeviceManagerClient,
                                                   handleOrPtrInfo);
  }

  function UsbDeviceManagerClientAssociatedPtr(associatedInterfacePtrInfo) {
    this.ptr = new associatedBindings.AssociatedInterfacePtrController(
        UsbDeviceManagerClient, associatedInterfacePtrInfo);
  }

  UsbDeviceManagerClientAssociatedPtr.prototype =
      Object.create(UsbDeviceManagerClientPtr.prototype);
  UsbDeviceManagerClientAssociatedPtr.prototype.constructor =
      UsbDeviceManagerClientAssociatedPtr;

  function UsbDeviceManagerClientProxy(receiver) {
    this.receiver_ = receiver;
  }
  UsbDeviceManagerClientPtr.prototype.onDeviceAdded = function() {
    return UsbDeviceManagerClientProxy.prototype.onDeviceAdded
        .apply(this.ptr.getProxy(), arguments);
  };

  UsbDeviceManagerClientProxy.prototype.onDeviceAdded = function(deviceInfo) {
    var params_ = new UsbDeviceManagerClient_OnDeviceAdded_Params();
    params_.deviceInfo = deviceInfo;
    var builder = new codec.MessageV0Builder(
        kUsbDeviceManagerClient_OnDeviceAdded_Name,
        codec.align(UsbDeviceManagerClient_OnDeviceAdded_Params.encodedSize));
    builder.encodeStruct(UsbDeviceManagerClient_OnDeviceAdded_Params, params_);
    var message = builder.finish();
    this.receiver_.accept(message);
  };
  UsbDeviceManagerClientPtr.prototype.onDeviceRemoved = function() {
    return UsbDeviceManagerClientProxy.prototype.onDeviceRemoved
        .apply(this.ptr.getProxy(), arguments);
  };

  UsbDeviceManagerClientProxy.prototype.onDeviceRemoved = function(deviceInfo) {
    var params_ = new UsbDeviceManagerClient_OnDeviceRemoved_Params();
    params_.deviceInfo = deviceInfo;
    var builder = new codec.MessageV0Builder(
        kUsbDeviceManagerClient_OnDeviceRemoved_Name,
        codec.align(UsbDeviceManagerClient_OnDeviceRemoved_Params.encodedSize));
    builder.encodeStruct(UsbDeviceManagerClient_OnDeviceRemoved_Params, params_);
    var message = builder.finish();
    this.receiver_.accept(message);
  };

  function UsbDeviceManagerClientStub(delegate) {
    this.delegate_ = delegate;
  }
  UsbDeviceManagerClientStub.prototype.onDeviceAdded = function(deviceInfo) {
    return this.delegate_ && this.delegate_.onDeviceAdded && this.delegate_.onDeviceAdded(deviceInfo);
  }
  UsbDeviceManagerClientStub.prototype.onDeviceRemoved = function(deviceInfo) {
    return this.delegate_ && this.delegate_.onDeviceRemoved && this.delegate_.onDeviceRemoved(deviceInfo);
  }

  UsbDeviceManagerClientStub.prototype.accept = function(message) {
    var reader = new codec.MessageReader(message);
    switch (reader.messageName) {
    case kUsbDeviceManagerClient_OnDeviceAdded_Name:
      var params = reader.decodeStruct(UsbDeviceManagerClient_OnDeviceAdded_Params);
      this.onDeviceAdded(params.deviceInfo);
      return true;
    case kUsbDeviceManagerClient_OnDeviceRemoved_Name:
      var params = reader.decodeStruct(UsbDeviceManagerClient_OnDeviceRemoved_Params);
      this.onDeviceRemoved(params.deviceInfo);
      return true;
    default:
      return false;
    }
  };

  UsbDeviceManagerClientStub.prototype.acceptWithResponder =
      function(message, responder) {
    var reader = new codec.MessageReader(message);
    switch (reader.messageName) {
    default:
      return false;
    }
  };

  function validateUsbDeviceManagerClientRequest(messageValidator) {
    var message = messageValidator.message;
    var paramsClass = null;
    switch (message.getName()) {
      case kUsbDeviceManagerClient_OnDeviceAdded_Name:
        if (!message.expectsResponse() && !message.isResponse())
          paramsClass = UsbDeviceManagerClient_OnDeviceAdded_Params;
      break;
      case kUsbDeviceManagerClient_OnDeviceRemoved_Name:
        if (!message.expectsResponse() && !message.isResponse())
          paramsClass = UsbDeviceManagerClient_OnDeviceRemoved_Params;
      break;
    }
    if (paramsClass === null)
      return validator.validationError.NONE;
    return paramsClass.validate(messageValidator, messageValidator.message.getHeaderNumBytes());
  }

  function validateUsbDeviceManagerClientResponse(messageValidator) {
    return validator.validationError.NONE;
  }

  var UsbDeviceManagerClient = {
    name: 'device.mojom.UsbDeviceManagerClient',
    kVersion: 0,
    ptrClass: UsbDeviceManagerClientPtr,
    proxyClass: UsbDeviceManagerClientProxy,
    stubClass: UsbDeviceManagerClientStub,
    validateRequest: validateUsbDeviceManagerClientRequest,
    validateResponse: null,
  };
  UsbDeviceManagerClientStub.prototype.validator = validateUsbDeviceManagerClientRequest;
  UsbDeviceManagerClientProxy.prototype.validator = null;
  exports.UsbDeviceManagerClient = UsbDeviceManagerClient;
  exports.UsbDeviceManagerClientPtr = UsbDeviceManagerClientPtr;
  exports.UsbDeviceManagerClientAssociatedPtr = UsbDeviceManagerClientAssociatedPtr;
})();