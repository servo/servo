// Copyright 2014 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

'use strict';

(function() {
  var mojomId = 'device/usb/public/mojom/chooser_service.mojom';
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
  var device_manager$ =
      mojo.internal.exposeNamespace('device.mojom');
  if (mojo.config.autoLoadMojomDeps) {
    mojo.internal.loadMojomIfNecessary(
        'device/usb/public/mojom/device_manager.mojom', 'device_manager.mojom.js');
  }



  function UsbChooserService_GetPermission_Params(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  UsbChooserService_GetPermission_Params.prototype.initDefaults_ = function() {
    this.deviceFilters = null;
  };
  UsbChooserService_GetPermission_Params.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  UsbChooserService_GetPermission_Params.validate = function(messageValidator, offset) {
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


    // validate UsbChooserService_GetPermission_Params.deviceFilters
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 0, 8, new codec.PointerTo(device_manager$.UsbDeviceFilter), false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  UsbChooserService_GetPermission_Params.encodedSize = codec.kStructHeaderSize + 8;

  UsbChooserService_GetPermission_Params.decode = function(decoder) {
    var packed;
    var val = new UsbChooserService_GetPermission_Params();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.deviceFilters = decoder.decodeArrayPointer(new codec.PointerTo(device_manager$.UsbDeviceFilter));
    return val;
  };

  UsbChooserService_GetPermission_Params.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(UsbChooserService_GetPermission_Params.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeArrayPointer(new codec.PointerTo(device_manager$.UsbDeviceFilter), val.deviceFilters);
  };
  function UsbChooserService_GetPermission_ResponseParams(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  UsbChooserService_GetPermission_ResponseParams.prototype.initDefaults_ = function() {
    this.result = null;
  };
  UsbChooserService_GetPermission_ResponseParams.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  UsbChooserService_GetPermission_ResponseParams.validate = function(messageValidator, offset) {
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


    // validate UsbChooserService_GetPermission_ResponseParams.result
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 0, device$.UsbDeviceInfo, true);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  UsbChooserService_GetPermission_ResponseParams.encodedSize = codec.kStructHeaderSize + 8;

  UsbChooserService_GetPermission_ResponseParams.decode = function(decoder) {
    var packed;
    var val = new UsbChooserService_GetPermission_ResponseParams();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.result = decoder.decodeStructPointer(device$.UsbDeviceInfo);
    return val;
  };

  UsbChooserService_GetPermission_ResponseParams.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(UsbChooserService_GetPermission_ResponseParams.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStructPointer(device$.UsbDeviceInfo, val.result);
  };
  var kUsbChooserService_GetPermission_Name = 0;

  function UsbChooserServicePtr(handleOrPtrInfo) {
    this.ptr = new bindings.InterfacePtrController(UsbChooserService,
                                                   handleOrPtrInfo);
  }

  function UsbChooserServiceAssociatedPtr(associatedInterfacePtrInfo) {
    this.ptr = new associatedBindings.AssociatedInterfacePtrController(
        UsbChooserService, associatedInterfacePtrInfo);
  }

  UsbChooserServiceAssociatedPtr.prototype =
      Object.create(UsbChooserServicePtr.prototype);
  UsbChooserServiceAssociatedPtr.prototype.constructor =
      UsbChooserServiceAssociatedPtr;

  function UsbChooserServiceProxy(receiver) {
    this.receiver_ = receiver;
  }
  UsbChooserServicePtr.prototype.getPermission = function() {
    return UsbChooserServiceProxy.prototype.getPermission
        .apply(this.ptr.getProxy(), arguments);
  };

  UsbChooserServiceProxy.prototype.getPermission = function(deviceFilters) {
    var params = new UsbChooserService_GetPermission_Params();
    params.deviceFilters = deviceFilters;
    return new Promise(function(resolve, reject) {
      var builder = new codec.MessageV1Builder(
          kUsbChooserService_GetPermission_Name,
          codec.align(UsbChooserService_GetPermission_Params.encodedSize),
          codec.kMessageExpectsResponse, 0);
      builder.encodeStruct(UsbChooserService_GetPermission_Params, params);
      var message = builder.finish();
      this.receiver_.acceptAndExpectResponse(message).then(function(message) {
        var reader = new codec.MessageReader(message);
        var responseParams =
            reader.decodeStruct(UsbChooserService_GetPermission_ResponseParams);
        resolve(responseParams);
      }).catch(function(result) {
        reject(Error("Connection error: " + result));
      });
    }.bind(this));
  };

  function UsbChooserServiceStub(delegate) {
    this.delegate_ = delegate;
  }
  UsbChooserServiceStub.prototype.getPermission = function(deviceFilters) {
    return this.delegate_ && this.delegate_.getPermission && this.delegate_.getPermission(deviceFilters);
  }

  UsbChooserServiceStub.prototype.accept = function(message) {
    var reader = new codec.MessageReader(message);
    switch (reader.messageName) {
    default:
      return false;
    }
  };

  UsbChooserServiceStub.prototype.acceptWithResponder =
      function(message, responder) {
    var reader = new codec.MessageReader(message);
    switch (reader.messageName) {
    case kUsbChooserService_GetPermission_Name:
      var params = reader.decodeStruct(UsbChooserService_GetPermission_Params);
      this.getPermission(params.deviceFilters).then(function(response) {
        var responseParams =
            new UsbChooserService_GetPermission_ResponseParams();
        responseParams.result = response.result;
        var builder = new codec.MessageV1Builder(
            kUsbChooserService_GetPermission_Name,
            codec.align(UsbChooserService_GetPermission_ResponseParams.encodedSize),
            codec.kMessageIsResponse, reader.requestID);
        builder.encodeStruct(UsbChooserService_GetPermission_ResponseParams,
                             responseParams);
        var message = builder.finish();
        responder.accept(message);
      });
      return true;
    default:
      return false;
    }
  };

  function validateUsbChooserServiceRequest(messageValidator) {
    var message = messageValidator.message;
    var paramsClass = null;
    switch (message.getName()) {
      case kUsbChooserService_GetPermission_Name:
        if (message.expectsResponse())
          paramsClass = UsbChooserService_GetPermission_Params;
      break;
    }
    if (paramsClass === null)
      return validator.validationError.NONE;
    return paramsClass.validate(messageValidator, messageValidator.message.getHeaderNumBytes());
  }

  function validateUsbChooserServiceResponse(messageValidator) {
   var message = messageValidator.message;
   var paramsClass = null;
   switch (message.getName()) {
      case kUsbChooserService_GetPermission_Name:
        if (message.isResponse())
          paramsClass = UsbChooserService_GetPermission_ResponseParams;
        break;
    }
    if (paramsClass === null)
      return validator.validationError.NONE;
    return paramsClass.validate(messageValidator, messageValidator.message.getHeaderNumBytes());
  }

  var UsbChooserService = {
    name: 'device.mojom.UsbChooserService',
    kVersion: 0,
    ptrClass: UsbChooserServicePtr,
    proxyClass: UsbChooserServiceProxy,
    stubClass: UsbChooserServiceStub,
    validateRequest: validateUsbChooserServiceRequest,
    validateResponse: validateUsbChooserServiceResponse,
  };
  UsbChooserServiceStub.prototype.validator = validateUsbChooserServiceRequest;
  UsbChooserServiceProxy.prototype.validator = validateUsbChooserServiceResponse;
  exports.UsbChooserService = UsbChooserService;
  exports.UsbChooserServicePtr = UsbChooserServicePtr;
  exports.UsbChooserServiceAssociatedPtr = UsbChooserServiceAssociatedPtr;
})();
