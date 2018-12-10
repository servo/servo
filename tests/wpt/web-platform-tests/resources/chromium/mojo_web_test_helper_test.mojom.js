// Copyright 2014 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

'use strict';

(function() {
  var mojomId = 'content/test/data/mojo_web_test_helper_test.mojom';
  if (mojo.internal.isMojomLoaded(mojomId)) {
    console.warn('The following mojom is loaded multiple times: ' + mojomId);
    return;
  }
  mojo.internal.markMojomLoaded(mojomId);

  // TODO(yzshen): Define these aliases to minimize the differences between the
  // old/new modes. Remove them when the old mode goes away.
  var bindings = mojo;
  var associatedBindings = mojo;
  var codec = mojo.internal;
  var validator = mojo.internal;

  var exports = mojo.internal.exposeNamespace('content.mojom');



  function MojoWebTestHelper_Reverse_Params(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  MojoWebTestHelper_Reverse_Params.prototype.initDefaults_ = function() {
    this.message = null;
  };
  MojoWebTestHelper_Reverse_Params.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  MojoWebTestHelper_Reverse_Params.validate = function(messageValidator, offset) {
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


    // validate MojoWebTestHelper_Reverse_Params.message
    err = messageValidator.validateStringPointer(offset + codec.kStructHeaderSize + 0, false)
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  MojoWebTestHelper_Reverse_Params.encodedSize = codec.kStructHeaderSize + 8;

  MojoWebTestHelper_Reverse_Params.decode = function(decoder) {
    var packed;
    var val = new MojoWebTestHelper_Reverse_Params();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.message = decoder.decodeStruct(codec.String);
    return val;
  };

  MojoWebTestHelper_Reverse_Params.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(MojoWebTestHelper_Reverse_Params.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.String, val.message);
  };
  function MojoWebTestHelper_Reverse_ResponseParams(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  MojoWebTestHelper_Reverse_ResponseParams.prototype.initDefaults_ = function() {
    this.reversed = null;
  };
  MojoWebTestHelper_Reverse_ResponseParams.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  MojoWebTestHelper_Reverse_ResponseParams.validate = function(messageValidator, offset) {
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


    // validate MojoWebTestHelper_Reverse_ResponseParams.reversed
    err = messageValidator.validateStringPointer(offset + codec.kStructHeaderSize + 0, false)
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  MojoWebTestHelper_Reverse_ResponseParams.encodedSize = codec.kStructHeaderSize + 8;

  MojoWebTestHelper_Reverse_ResponseParams.decode = function(decoder) {
    var packed;
    var val = new MojoWebTestHelper_Reverse_ResponseParams();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.reversed = decoder.decodeStruct(codec.String);
    return val;
  };

  MojoWebTestHelper_Reverse_ResponseParams.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(MojoWebTestHelper_Reverse_ResponseParams.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.String, val.reversed);
  };
  var kMojoWebTestHelper_Reverse_Name = 0;

  function MojoWebTestHelperPtr(handleOrPtrInfo) {
    this.ptr = new bindings.InterfacePtrController(MojoWebTestHelper,
                                                   handleOrPtrInfo);
  }

  function MojoWebTestHelperAssociatedPtr(associatedInterfacePtrInfo) {
    this.ptr = new associatedBindings.AssociatedInterfacePtrController(
        MojoWebTestHelper, associatedInterfacePtrInfo);
  }

  MojoWebTestHelperAssociatedPtr.prototype =
      Object.create(MojoWebTestHelperPtr.prototype);
  MojoWebTestHelperAssociatedPtr.prototype.constructor =
      MojoWebTestHelperAssociatedPtr;

  function MojoWebTestHelperProxy(receiver) {
    this.receiver_ = receiver;
  }
  MojoWebTestHelperPtr.prototype.reverse = function() {
    return MojoWebTestHelperProxy.prototype.reverse
        .apply(this.ptr.getProxy(), arguments);
  };

  MojoWebTestHelperProxy.prototype.reverse = function(message) {
    var params = new MojoWebTestHelper_Reverse_Params();
    params.message = message;
    return new Promise(function(resolve, reject) {
      var builder = new codec.MessageV1Builder(
          kMojoWebTestHelper_Reverse_Name,
          codec.align(MojoWebTestHelper_Reverse_Params.encodedSize),
          codec.kMessageExpectsResponse, 0);
      builder.encodeStruct(MojoWebTestHelper_Reverse_Params, params);
      var message = builder.finish();
      this.receiver_.acceptAndExpectResponse(message).then(function(message) {
        var reader = new codec.MessageReader(message);
        var responseParams =
            reader.decodeStruct(MojoWebTestHelper_Reverse_ResponseParams);
        resolve(responseParams);
      }).catch(function(result) {
        reject(Error("Connection error: " + result));
      });
    }.bind(this));
  };

  function MojoWebTestHelperStub(delegate) {
    this.delegate_ = delegate;
  }
  MojoWebTestHelperStub.prototype.reverse = function(message) {
    return this.delegate_ && this.delegate_.reverse && this.delegate_.reverse(message);
  }

  MojoWebTestHelperStub.prototype.accept = function(message) {
    var reader = new codec.MessageReader(message);
    switch (reader.messageName) {
    default:
      return false;
    }
  };

  MojoWebTestHelperStub.prototype.acceptWithResponder =
      function(message, responder) {
    var reader = new codec.MessageReader(message);
    switch (reader.messageName) {
    case kMojoWebTestHelper_Reverse_Name:
      var params = reader.decodeStruct(MojoWebTestHelper_Reverse_Params);
      this.reverse(params.message).then(function(response) {
        var responseParams =
            new MojoWebTestHelper_Reverse_ResponseParams();
        responseParams.reversed = response.reversed;
        var builder = new codec.MessageV1Builder(
            kMojoWebTestHelper_Reverse_Name,
            codec.align(MojoWebTestHelper_Reverse_ResponseParams.encodedSize),
            codec.kMessageIsResponse, reader.requestID);
        builder.encodeStruct(MojoWebTestHelper_Reverse_ResponseParams,
                             responseParams);
        var message = builder.finish();
        responder.accept(message);
      });
      return true;
    default:
      return false;
    }
  };

  function validateMojoWebTestHelperRequest(messageValidator) {
    var message = messageValidator.message;
    var paramsClass = null;
    switch (message.getName()) {
      case kMojoWebTestHelper_Reverse_Name:
        if (message.expectsResponse())
          paramsClass = MojoWebTestHelper_Reverse_Params;
      break;
    }
    if (paramsClass === null)
      return validator.validationError.NONE;
    return paramsClass.validate(messageValidator, messageValidator.message.getHeaderNumBytes());
  }

  function validateMojoWebTestHelperResponse(messageValidator) {
   var message = messageValidator.message;
   var paramsClass = null;
   switch (message.getName()) {
      case kMojoWebTestHelper_Reverse_Name:
        if (message.isResponse())
          paramsClass = MojoWebTestHelper_Reverse_ResponseParams;
        break;
    }
    if (paramsClass === null)
      return validator.validationError.NONE;
    return paramsClass.validate(messageValidator, messageValidator.message.getHeaderNumBytes());
  }

  var MojoWebTestHelper = {
    name: 'content.mojom.MojoWebTestHelper',
    kVersion: 0,
    ptrClass: MojoWebTestHelperPtr,
    proxyClass: MojoWebTestHelperProxy,
    stubClass: MojoWebTestHelperStub,
    validateRequest: validateMojoWebTestHelperRequest,
    validateResponse: validateMojoWebTestHelperResponse,
  };
  MojoWebTestHelperStub.prototype.validator = validateMojoWebTestHelperRequest;
  MojoWebTestHelperProxy.prototype.validator = validateMojoWebTestHelperResponse;
  exports.MojoWebTestHelper = MojoWebTestHelper;
  exports.MojoWebTestHelperPtr = MojoWebTestHelperPtr;
  exports.MojoWebTestHelperAssociatedPtr = MojoWebTestHelperAssociatedPtr;
})();
