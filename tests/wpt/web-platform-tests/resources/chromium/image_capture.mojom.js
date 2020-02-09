// Copyright 2014 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

'use strict';

(function() {
  var mojomId = 'media/capture/mojom/image_capture.mojom';
  if (mojo.internal.isMojomLoaded(mojomId)) {
    console.warn('The following mojom is loaded multiple times: ' + mojomId);
    return;
  }
  mojo.internal.markMojomLoaded(mojomId);
  var bindings = mojo;
  var associatedBindings = mojo;
  var codec = mojo.internal;
  var validator = mojo.internal;

  var exports = mojo.internal.exposeNamespace('media.mojom');


  var MeteringMode = {};
  MeteringMode.NONE = 0;
  MeteringMode.MANUAL = MeteringMode.NONE + 1;
  MeteringMode.SINGLE_SHOT = MeteringMode.MANUAL + 1;
  MeteringMode.CONTINUOUS = MeteringMode.SINGLE_SHOT + 1;
  MeteringMode.MIN_VALUE = 0,
  MeteringMode.MAX_VALUE = 3,

  MeteringMode.isKnownEnumValue = function(value) {
    switch (value) {
    case 0:
    case 1:
    case 2:
    case 3:
      return true;
    }
    return false;
  };

  MeteringMode.validate = function(enumValue) {
    var isExtensible = false;
    if (isExtensible || this.isKnownEnumValue(enumValue))
      return validator.validationError.NONE;

    return validator.validationError.UNKNOWN_ENUM_VALUE;
  };
  var RedEyeReduction = {};
  RedEyeReduction.NEVER = 0;
  RedEyeReduction.ALWAYS = RedEyeReduction.NEVER + 1;
  RedEyeReduction.CONTROLLABLE = RedEyeReduction.ALWAYS + 1;
  RedEyeReduction.MIN_VALUE = 0,
  RedEyeReduction.MAX_VALUE = 2,

  RedEyeReduction.isKnownEnumValue = function(value) {
    switch (value) {
    case 0:
    case 1:
    case 2:
      return true;
    }
    return false;
  };

  RedEyeReduction.validate = function(enumValue) {
    var isExtensible = false;
    if (isExtensible || this.isKnownEnumValue(enumValue))
      return validator.validationError.NONE;

    return validator.validationError.UNKNOWN_ENUM_VALUE;
  };
  var FillLightMode = {};
  FillLightMode.OFF = 0;
  FillLightMode.AUTO = FillLightMode.OFF + 1;
  FillLightMode.FLASH = FillLightMode.AUTO + 1;
  FillLightMode.MIN_VALUE = 0,
  FillLightMode.MAX_VALUE = 2,

  FillLightMode.isKnownEnumValue = function(value) {
    switch (value) {
    case 0:
    case 1:
    case 2:
      return true;
    }
    return false;
  };

  FillLightMode.validate = function(enumValue) {
    var isExtensible = false;
    if (isExtensible || this.isKnownEnumValue(enumValue))
      return validator.validationError.NONE;

    return validator.validationError.UNKNOWN_ENUM_VALUE;
  };

  function Range(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  Range.prototype.initDefaults_ = function() {
    this.max = 0;
    this.min = 0;
    this.current = 0;
    this.step = 0;
  };
  Range.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  Range.validate = function(messageValidator, offset) {
    var err;
    err = messageValidator.validateStructHeader(offset, codec.kStructHeaderSize);
    if (err !== validator.validationError.NONE)
        return err;

    var kVersionSizes = [
      {version: 0, numBytes: 40}
    ];
    err = messageValidator.validateStructVersion(offset, kVersionSizes);
    if (err !== validator.validationError.NONE)
        return err;





    return validator.validationError.NONE;
  };

  Range.encodedSize = codec.kStructHeaderSize + 32;

  Range.decode = function(decoder) {
    var packed;
    var val = new Range();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.max = decoder.decodeStruct(codec.Double);
    val.min = decoder.decodeStruct(codec.Double);
    val.current = decoder.decodeStruct(codec.Double);
    val.step = decoder.decodeStruct(codec.Double);
    return val;
  };

  Range.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(Range.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.Double, val.max);
    encoder.encodeStruct(codec.Double, val.min);
    encoder.encodeStruct(codec.Double, val.current);
    encoder.encodeStruct(codec.Double, val.step);
  };
  function PhotoState(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  PhotoState.prototype.initDefaults_ = function() {
    this.supportedWhiteBalanceModes = null;
    this.currentWhiteBalanceMode = 0;
    this.currentExposureMode = 0;
    this.supportedExposureModes = null;
    this.supportedFocusModes = null;
    this.currentFocusMode = 0;
    this.supportsTorch = false;
    this.torch = false;
    this.pointsOfInterest = null;
    this.exposureCompensation = null;
    this.exposureTime = null;
    this.colorTemperature = null;
    this.iso = null;
    this.brightness = null;
    this.contrast = null;
    this.saturation = null;
    this.sharpness = null;
    this.focusDistance = null;
    this.pan = null;
    this.tilt = null;
    this.zoom = null;
    this.redEyeReduction = 0;
    this.height = null;
    this.width = null;
    this.fillLightMode = null;
  };
  PhotoState.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  PhotoState.validate = function(messageValidator, offset) {
    var err;
    err = messageValidator.validateStructHeader(offset, codec.kStructHeaderSize);
    if (err !== validator.validationError.NONE)
        return err;

    var kVersionSizes = [
      {version: 0, numBytes: 184}
    ];
    err = messageValidator.validateStructVersion(offset, kVersionSizes);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.supportedWhiteBalanceModes
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 0, 4, new codec.Enum(MeteringMode), false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.currentWhiteBalanceMode
    err = messageValidator.validateEnum(offset + codec.kStructHeaderSize + 8, MeteringMode);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.supportedExposureModes
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 16, 4, new codec.Enum(MeteringMode), false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.currentExposureMode
    err = messageValidator.validateEnum(offset + codec.kStructHeaderSize + 12, MeteringMode);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.supportedFocusModes
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 24, 4, new codec.Enum(MeteringMode), false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.currentFocusMode
    err = messageValidator.validateEnum(offset + codec.kStructHeaderSize + 32, MeteringMode);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.pointsOfInterest
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 40, 8, new codec.PointerTo(Point2D), false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.exposureCompensation
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 48, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.exposureTime
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 56, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.colorTemperature
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 64, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.iso
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 72, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.brightness
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 80, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.contrast
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 88, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.saturation
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 96, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.sharpness
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 104, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.focusDistance
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 112, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.pan
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 120, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.tilt
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 128, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.zoom
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 136, Range, false);
    if (err !== validator.validationError.NONE)
        return err;




    // validate PhotoState.redEyeReduction
    err = messageValidator.validateEnum(offset + codec.kStructHeaderSize + 144, RedEyeReduction);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.height
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 152, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.width
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 160, Range, false);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoState.fillLightMode
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 168, 4, new codec.Enum(FillLightMode), false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  PhotoState.encodedSize = codec.kStructHeaderSize + 176;

  PhotoState.decode = function(decoder) {
    var packed;
    var val = new PhotoState();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.supportedWhiteBalanceModes = decoder.decodeArrayPointer(new codec.Enum(MeteringMode));
    val.currentWhiteBalanceMode = decoder.decodeStruct(codec.Int32);
    val.currentExposureMode = decoder.decodeStruct(codec.Int32);
    val.supportedExposureModes = decoder.decodeArrayPointer(new codec.Enum(MeteringMode));
    val.supportedFocusModes = decoder.decodeArrayPointer(new codec.Enum(MeteringMode));
    val.currentFocusMode = decoder.decodeStruct(codec.Int32);
    packed = decoder.readUint8();
    val.supportsTorch = (packed >> 0) & 1 ? true : false;
    val.torch = (packed >> 1) & 1 ? true : false;
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    val.pointsOfInterest = decoder.decodeArrayPointer(new codec.PointerTo(Point2D));
    val.exposureCompensation = decoder.decodeStructPointer(Range);
    val.exposureTime = decoder.decodeStructPointer(Range);
    val.colorTemperature = decoder.decodeStructPointer(Range);
    val.iso = decoder.decodeStructPointer(Range);
    val.brightness = decoder.decodeStructPointer(Range);
    val.contrast = decoder.decodeStructPointer(Range);
    val.saturation = decoder.decodeStructPointer(Range);
    val.sharpness = decoder.decodeStructPointer(Range);
    val.focusDistance = decoder.decodeStructPointer(Range);
    val.pan = decoder.decodeStructPointer(Range);
    val.tilt = decoder.decodeStructPointer(Range);
    val.zoom = decoder.decodeStructPointer(Range);
    val.redEyeReduction = decoder.decodeStruct(codec.Int32);
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    val.height = decoder.decodeStructPointer(Range);
    val.width = decoder.decodeStructPointer(Range);
    val.fillLightMode = decoder.decodeArrayPointer(new codec.Enum(FillLightMode));
    return val;
  };

  PhotoState.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(PhotoState.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeArrayPointer(new codec.Enum(MeteringMode), val.supportedWhiteBalanceModes);
    encoder.encodeStruct(codec.Int32, val.currentWhiteBalanceMode);
    encoder.encodeStruct(codec.Int32, val.currentExposureMode);
    encoder.encodeArrayPointer(new codec.Enum(MeteringMode), val.supportedExposureModes);
    encoder.encodeArrayPointer(new codec.Enum(MeteringMode), val.supportedFocusModes);
    encoder.encodeStruct(codec.Int32, val.currentFocusMode);
    packed = 0;
    packed |= (val.supportsTorch & 1) << 0
    packed |= (val.torch & 1) << 1
    encoder.writeUint8(packed);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.encodeArrayPointer(new codec.PointerTo(Point2D), val.pointsOfInterest);
    encoder.encodeStructPointer(Range, val.exposureCompensation);
    encoder.encodeStructPointer(Range, val.exposureTime);
    encoder.encodeStructPointer(Range, val.colorTemperature);
    encoder.encodeStructPointer(Range, val.iso);
    encoder.encodeStructPointer(Range, val.brightness);
    encoder.encodeStructPointer(Range, val.contrast);
    encoder.encodeStructPointer(Range, val.saturation);
    encoder.encodeStructPointer(Range, val.sharpness);
    encoder.encodeStructPointer(Range, val.focusDistance);
    encoder.encodeStructPointer(Range, val.pan);
    encoder.encodeStructPointer(Range, val.tilt);
    encoder.encodeStructPointer(Range, val.zoom);
    encoder.encodeStruct(codec.Int32, val.redEyeReduction);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.encodeStructPointer(Range, val.height);
    encoder.encodeStructPointer(Range, val.width);
    encoder.encodeArrayPointer(new codec.Enum(FillLightMode), val.fillLightMode);
  };
  function Point2D(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  Point2D.prototype.initDefaults_ = function() {
    this.x = 0;
    this.y = 0;
  };
  Point2D.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  Point2D.validate = function(messageValidator, offset) {
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



    return validator.validationError.NONE;
  };

  Point2D.encodedSize = codec.kStructHeaderSize + 16;

  Point2D.decode = function(decoder) {
    var packed;
    var val = new Point2D();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.x = decoder.decodeStruct(codec.Double);
    val.y = decoder.decodeStruct(codec.Double);
    return val;
  };

  Point2D.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(Point2D.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.Double, val.x);
    encoder.encodeStruct(codec.Double, val.y);
  };
  function PhotoSettings(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  PhotoSettings.prototype.initDefaults_ = function() {
    this.hasWhiteBalanceMode = false;
    this.hasExposureMode = false;
    this.hasFocusMode = false;
    this.hasExposureCompensation = false;
    this.hasExposureTime = false;
    this.hasColorTemperature = false;
    this.hasIso = false;
    this.hasBrightness = false;
    this.hasContrast = false;
    this.hasSaturation = false;
    this.hasSharpness = false;
    this.hasFocusDistance = false;
    this.hasPan = false;
    this.hasTilt = false;
    this.hasZoom = false;
    this.hasTorch = false;
    this.torch = false;
    this.hasFillLightMode = false;
    this.hasWidth = false;
    this.hasHeight = false;
    this.hasRedEyeReduction = false;
    this.redEyeReduction = false;
    this.whiteBalanceMode = 0;
    this.exposureMode = 0;
    this.focusMode = 0;
    this.pointsOfInterest = null;
    this.exposureCompensation = 0;
    this.exposureTime = 0;
    this.colorTemperature = 0;
    this.iso = 0;
    this.brightness = 0;
    this.contrast = 0;
    this.saturation = 0;
    this.sharpness = 0;
    this.focusDistance = 0;
    this.pan = 0;
    this.tilt = 0;
    this.zoom = 0;
    this.fillLightMode = 0;
    this.width = 0;
    this.height = 0;
  };
  PhotoSettings.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  PhotoSettings.validate = function(messageValidator, offset) {
    var err;
    err = messageValidator.validateStructHeader(offset, codec.kStructHeaderSize);
    if (err !== validator.validationError.NONE)
        return err;

    var kVersionSizes = [
      {version: 0, numBytes: 152}
    ];
    err = messageValidator.validateStructVersion(offset, kVersionSizes);
    if (err !== validator.validationError.NONE)
        return err;



    // validate PhotoSettings.whiteBalanceMode
    err = messageValidator.validateEnum(offset + codec.kStructHeaderSize + 4, MeteringMode);
    if (err !== validator.validationError.NONE)
        return err;



    // validate PhotoSettings.exposureMode
    err = messageValidator.validateEnum(offset + codec.kStructHeaderSize + 8, MeteringMode);
    if (err !== validator.validationError.NONE)
        return err;



    // validate PhotoSettings.focusMode
    err = messageValidator.validateEnum(offset + codec.kStructHeaderSize + 12, MeteringMode);
    if (err !== validator.validationError.NONE)
        return err;


    // validate PhotoSettings.pointsOfInterest
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 16, 8, new codec.PointerTo(Point2D), false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;





























    // validate PhotoSettings.fillLightMode
    err = messageValidator.validateEnum(offset + codec.kStructHeaderSize + 120, FillLightMode);
    if (err !== validator.validationError.NONE)
        return err;







    return validator.validationError.NONE;
  };

  PhotoSettings.encodedSize = codec.kStructHeaderSize + 144;

  PhotoSettings.decode = function(decoder) {
    var packed;
    var val = new PhotoSettings();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    packed = decoder.readUint8();
    val.hasWhiteBalanceMode = (packed >> 0) & 1 ? true : false;
    val.hasExposureMode = (packed >> 1) & 1 ? true : false;
    val.hasFocusMode = (packed >> 2) & 1 ? true : false;
    val.hasExposureCompensation = (packed >> 3) & 1 ? true : false;
    val.hasExposureTime = (packed >> 4) & 1 ? true : false;
    val.hasColorTemperature = (packed >> 5) & 1 ? true : false;
    val.hasIso = (packed >> 6) & 1 ? true : false;
    val.hasBrightness = (packed >> 7) & 1 ? true : false;
    packed = decoder.readUint8();
    val.hasContrast = (packed >> 0) & 1 ? true : false;
    val.hasSaturation = (packed >> 1) & 1 ? true : false;
    val.hasSharpness = (packed >> 2) & 1 ? true : false;
    val.hasFocusDistance = (packed >> 3) & 1 ? true : false;
    val.hasPan = (packed >> 4) & 1 ? true : false;
    val.hasTilt = (packed >> 5) & 1 ? true : false;
    val.hasZoom = (packed >> 6) & 1 ? true : false;
    val.hasTorch = (packed >> 7) & 1 ? true : false;
    packed = decoder.readUint8();
    val.torch = (packed >> 0) & 1 ? true : false;
    val.hasFillLightMode = (packed >> 1) & 1 ? true : false;
    val.hasWidth = (packed >> 2) & 1 ? true : false;
    val.hasHeight = (packed >> 3) & 1 ? true : false;
    val.hasRedEyeReduction = (packed >> 4) & 1 ? true : false;
    val.redEyeReduction = (packed >> 5) & 1 ? true : false;
    decoder.skip(1);
    val.whiteBalanceMode = decoder.decodeStruct(codec.Int32);
    val.exposureMode = decoder.decodeStruct(codec.Int32);
    val.focusMode = decoder.decodeStruct(codec.Int32);
    val.pointsOfInterest = decoder.decodeArrayPointer(new codec.PointerTo(Point2D));
    val.exposureCompensation = decoder.decodeStruct(codec.Double);
    val.exposureTime = decoder.decodeStruct(codec.Double);
    val.colorTemperature = decoder.decodeStruct(codec.Double);
    val.iso = decoder.decodeStruct(codec.Double);
    val.brightness = decoder.decodeStruct(codec.Double);
    val.contrast = decoder.decodeStruct(codec.Double);
    val.saturation = decoder.decodeStruct(codec.Double);
    val.sharpness = decoder.decodeStruct(codec.Double);
    val.focusDistance = decoder.decodeStruct(codec.Double);
    val.pan = decoder.decodeStruct(codec.Double);
    val.tilt = decoder.decodeStruct(codec.Double);
    val.zoom = decoder.decodeStruct(codec.Double);
    val.fillLightMode = decoder.decodeStruct(codec.Int32);
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    val.width = decoder.decodeStruct(codec.Double);
    val.height = decoder.decodeStruct(codec.Double);
    return val;
  };

  PhotoSettings.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(PhotoSettings.encodedSize);
    encoder.writeUint32(0);
    packed = 0;
    packed |= (val.hasWhiteBalanceMode & 1) << 0
    packed |= (val.hasExposureMode & 1) << 1
    packed |= (val.hasFocusMode & 1) << 2
    packed |= (val.hasExposureCompensation & 1) << 3
    packed |= (val.hasExposureTime & 1) << 4
    packed |= (val.hasColorTemperature & 1) << 5
    packed |= (val.hasIso & 1) << 6
    packed |= (val.hasBrightness & 1) << 7
    encoder.writeUint8(packed);
    packed = 0;
    packed |= (val.hasContrast & 1) << 0
    packed |= (val.hasSaturation & 1) << 1
    packed |= (val.hasSharpness & 1) << 2
    packed |= (val.hasFocusDistance & 1) << 3
    packed |= (val.hasPan & 1) << 4
    packed |= (val.hasTilt & 1) << 5
    packed |= (val.hasZoom & 1) << 6
    packed |= (val.hasTorch & 1) << 7
    encoder.writeUint8(packed);
    packed = 0;
    packed |= (val.torch & 1) << 0
    packed |= (val.hasFillLightMode & 1) << 1
    packed |= (val.hasWidth & 1) << 2
    packed |= (val.hasHeight & 1) << 3
    packed |= (val.hasRedEyeReduction & 1) << 4
    packed |= (val.redEyeReduction & 1) << 5
    encoder.writeUint8(packed);
    encoder.skip(1);
    encoder.encodeStruct(codec.Int32, val.whiteBalanceMode);
    encoder.encodeStruct(codec.Int32, val.exposureMode);
    encoder.encodeStruct(codec.Int32, val.focusMode);
    encoder.encodeArrayPointer(new codec.PointerTo(Point2D), val.pointsOfInterest);
    encoder.encodeStruct(codec.Double, val.exposureCompensation);
    encoder.encodeStruct(codec.Double, val.exposureTime);
    encoder.encodeStruct(codec.Double, val.colorTemperature);
    encoder.encodeStruct(codec.Double, val.iso);
    encoder.encodeStruct(codec.Double, val.brightness);
    encoder.encodeStruct(codec.Double, val.contrast);
    encoder.encodeStruct(codec.Double, val.saturation);
    encoder.encodeStruct(codec.Double, val.sharpness);
    encoder.encodeStruct(codec.Double, val.focusDistance);
    encoder.encodeStruct(codec.Double, val.pan);
    encoder.encodeStruct(codec.Double, val.tilt);
    encoder.encodeStruct(codec.Double, val.zoom);
    encoder.encodeStruct(codec.Int32, val.fillLightMode);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.encodeStruct(codec.Double, val.width);
    encoder.encodeStruct(codec.Double, val.height);
  };
  function Blob(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  Blob.prototype.initDefaults_ = function() {
    this.mimeType = null;
    this.data = null;
  };
  Blob.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  Blob.validate = function(messageValidator, offset) {
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


    // validate Blob.mimeType
    err = messageValidator.validateStringPointer(offset + codec.kStructHeaderSize + 0, false)
    if (err !== validator.validationError.NONE)
        return err;


    // validate Blob.data
    err = messageValidator.validateArrayPointer(offset + codec.kStructHeaderSize + 8, 1, codec.Uint8, false, [0], 0);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  Blob.encodedSize = codec.kStructHeaderSize + 16;

  Blob.decode = function(decoder) {
    var packed;
    var val = new Blob();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.mimeType = decoder.decodeStruct(codec.String);
    val.data = decoder.decodeArrayPointer(codec.Uint8);
    return val;
  };

  Blob.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(Blob.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.String, val.mimeType);
    encoder.encodeArrayPointer(codec.Uint8, val.data);
  };
  function ImageCapture_GetPhotoState_Params(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  ImageCapture_GetPhotoState_Params.prototype.initDefaults_ = function() {
    this.sourceId = null;
  };
  ImageCapture_GetPhotoState_Params.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  ImageCapture_GetPhotoState_Params.validate = function(messageValidator, offset) {
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


    // validate ImageCapture_GetPhotoState_Params.sourceId
    err = messageValidator.validateStringPointer(offset + codec.kStructHeaderSize + 0, false)
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  ImageCapture_GetPhotoState_Params.encodedSize = codec.kStructHeaderSize + 8;

  ImageCapture_GetPhotoState_Params.decode = function(decoder) {
    var packed;
    var val = new ImageCapture_GetPhotoState_Params();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.sourceId = decoder.decodeStruct(codec.String);
    return val;
  };

  ImageCapture_GetPhotoState_Params.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(ImageCapture_GetPhotoState_Params.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.String, val.sourceId);
  };
  function ImageCapture_GetPhotoState_ResponseParams(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  ImageCapture_GetPhotoState_ResponseParams.prototype.initDefaults_ = function() {
    this.state = null;
  };
  ImageCapture_GetPhotoState_ResponseParams.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  ImageCapture_GetPhotoState_ResponseParams.validate = function(messageValidator, offset) {
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


    // validate ImageCapture_GetPhotoState_ResponseParams.state
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 0, PhotoState, false);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  ImageCapture_GetPhotoState_ResponseParams.encodedSize = codec.kStructHeaderSize + 8;

  ImageCapture_GetPhotoState_ResponseParams.decode = function(decoder) {
    var packed;
    var val = new ImageCapture_GetPhotoState_ResponseParams();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.state = decoder.decodeStructPointer(PhotoState);
    return val;
  };

  ImageCapture_GetPhotoState_ResponseParams.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(ImageCapture_GetPhotoState_ResponseParams.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStructPointer(PhotoState, val.state);
  };
  function ImageCapture_SetOptions_Params(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  ImageCapture_SetOptions_Params.prototype.initDefaults_ = function() {
    this.sourceId = null;
    this.settings = null;
  };
  ImageCapture_SetOptions_Params.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  ImageCapture_SetOptions_Params.validate = function(messageValidator, offset) {
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


    // validate ImageCapture_SetOptions_Params.sourceId
    err = messageValidator.validateStringPointer(offset + codec.kStructHeaderSize + 0, false)
    if (err !== validator.validationError.NONE)
        return err;


    // validate ImageCapture_SetOptions_Params.settings
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 8, PhotoSettings, false);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  ImageCapture_SetOptions_Params.encodedSize = codec.kStructHeaderSize + 16;

  ImageCapture_SetOptions_Params.decode = function(decoder) {
    var packed;
    var val = new ImageCapture_SetOptions_Params();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.sourceId = decoder.decodeStruct(codec.String);
    val.settings = decoder.decodeStructPointer(PhotoSettings);
    return val;
  };

  ImageCapture_SetOptions_Params.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(ImageCapture_SetOptions_Params.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.String, val.sourceId);
    encoder.encodeStructPointer(PhotoSettings, val.settings);
  };
  function ImageCapture_SetOptions_ResponseParams(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  ImageCapture_SetOptions_ResponseParams.prototype.initDefaults_ = function() {
    this.success = false;
  };
  ImageCapture_SetOptions_ResponseParams.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  ImageCapture_SetOptions_ResponseParams.validate = function(messageValidator, offset) {
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


    return validator.validationError.NONE;
  };

  ImageCapture_SetOptions_ResponseParams.encodedSize = codec.kStructHeaderSize + 8;

  ImageCapture_SetOptions_ResponseParams.decode = function(decoder) {
    var packed;
    var val = new ImageCapture_SetOptions_ResponseParams();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    packed = decoder.readUint8();
    val.success = (packed >> 0) & 1 ? true : false;
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    decoder.skip(1);
    return val;
  };

  ImageCapture_SetOptions_ResponseParams.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(ImageCapture_SetOptions_ResponseParams.encodedSize);
    encoder.writeUint32(0);
    packed = 0;
    packed |= (val.success & 1) << 0
    encoder.writeUint8(packed);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
    encoder.skip(1);
  };
  function ImageCapture_TakePhoto_Params(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  ImageCapture_TakePhoto_Params.prototype.initDefaults_ = function() {
    this.sourceId = null;
  };
  ImageCapture_TakePhoto_Params.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  ImageCapture_TakePhoto_Params.validate = function(messageValidator, offset) {
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


    // validate ImageCapture_TakePhoto_Params.sourceId
    err = messageValidator.validateStringPointer(offset + codec.kStructHeaderSize + 0, false)
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  ImageCapture_TakePhoto_Params.encodedSize = codec.kStructHeaderSize + 8;

  ImageCapture_TakePhoto_Params.decode = function(decoder) {
    var packed;
    var val = new ImageCapture_TakePhoto_Params();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.sourceId = decoder.decodeStruct(codec.String);
    return val;
  };

  ImageCapture_TakePhoto_Params.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(ImageCapture_TakePhoto_Params.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStruct(codec.String, val.sourceId);
  };
  function ImageCapture_TakePhoto_ResponseParams(values) {
    this.initDefaults_();
    this.initFields_(values);
  }


  ImageCapture_TakePhoto_ResponseParams.prototype.initDefaults_ = function() {
    this.blob = null;
  };
  ImageCapture_TakePhoto_ResponseParams.prototype.initFields_ = function(fields) {
    for(var field in fields) {
        if (this.hasOwnProperty(field))
          this[field] = fields[field];
    }
  };

  ImageCapture_TakePhoto_ResponseParams.validate = function(messageValidator, offset) {
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


    // validate ImageCapture_TakePhoto_ResponseParams.blob
    err = messageValidator.validateStructPointer(offset + codec.kStructHeaderSize + 0, Blob, false);
    if (err !== validator.validationError.NONE)
        return err;

    return validator.validationError.NONE;
  };

  ImageCapture_TakePhoto_ResponseParams.encodedSize = codec.kStructHeaderSize + 8;

  ImageCapture_TakePhoto_ResponseParams.decode = function(decoder) {
    var packed;
    var val = new ImageCapture_TakePhoto_ResponseParams();
    var numberOfBytes = decoder.readUint32();
    var version = decoder.readUint32();
    val.blob = decoder.decodeStructPointer(Blob);
    return val;
  };

  ImageCapture_TakePhoto_ResponseParams.encode = function(encoder, val) {
    var packed;
    encoder.writeUint32(ImageCapture_TakePhoto_ResponseParams.encodedSize);
    encoder.writeUint32(0);
    encoder.encodeStructPointer(Blob, val.blob);
  };
  var kImageCapture_GetPhotoState_Name = 0;
  var kImageCapture_SetOptions_Name = 1;
  var kImageCapture_TakePhoto_Name = 2;

  function ImageCapturePtr(handleOrPtrInfo) {
    this.ptr = new bindings.InterfacePtrController(ImageCapture,
                                                   handleOrPtrInfo);
  }

  function ImageCaptureAssociatedPtr(associatedInterfacePtrInfo) {
    this.ptr = new associatedBindings.AssociatedInterfacePtrController(
        ImageCapture, associatedInterfacePtrInfo);
  }

  ImageCaptureAssociatedPtr.prototype =
      Object.create(ImageCapturePtr.prototype);
  ImageCaptureAssociatedPtr.prototype.constructor =
      ImageCaptureAssociatedPtr;

  function ImageCaptureProxy(receiver) {
    this.receiver_ = receiver;
  }
  ImageCapturePtr.prototype.getPhotoState = function() {
    return ImageCaptureProxy.prototype.getPhotoState
        .apply(this.ptr.getProxy(), arguments);
  };

  ImageCaptureProxy.prototype.getPhotoState = function(sourceId) {
    var params_ = new ImageCapture_GetPhotoState_Params();
    params_.sourceId = sourceId;
    return new Promise(function(resolve, reject) {
      var builder = new codec.MessageV1Builder(
          kImageCapture_GetPhotoState_Name,
          codec.align(ImageCapture_GetPhotoState_Params.encodedSize),
          codec.kMessageExpectsResponse, 0);
      builder.encodeStruct(ImageCapture_GetPhotoState_Params, params_);
      var message = builder.finish();
      this.receiver_.acceptAndExpectResponse(message).then(function(message) {
        var reader = new codec.MessageReader(message);
        var responseParams =
            reader.decodeStruct(ImageCapture_GetPhotoState_ResponseParams);
        resolve(responseParams);
      }).catch(function(result) {
        reject(Error("Connection error: " + result));
      });
    }.bind(this));
  };
  ImageCapturePtr.prototype.setOptions = function() {
    return ImageCaptureProxy.prototype.setOptions
        .apply(this.ptr.getProxy(), arguments);
  };

  ImageCaptureProxy.prototype.setOptions = function(sourceId, settings) {
    var params_ = new ImageCapture_SetOptions_Params();
    params_.sourceId = sourceId;
    params_.settings = settings;
    return new Promise(function(resolve, reject) {
      var builder = new codec.MessageV1Builder(
          kImageCapture_SetOptions_Name,
          codec.align(ImageCapture_SetOptions_Params.encodedSize),
          codec.kMessageExpectsResponse, 0);
      builder.encodeStruct(ImageCapture_SetOptions_Params, params_);
      var message = builder.finish();
      this.receiver_.acceptAndExpectResponse(message).then(function(message) {
        var reader = new codec.MessageReader(message);
        var responseParams =
            reader.decodeStruct(ImageCapture_SetOptions_ResponseParams);
        resolve(responseParams);
      }).catch(function(result) {
        reject(Error("Connection error: " + result));
      });
    }.bind(this));
  };
  ImageCapturePtr.prototype.takePhoto = function() {
    return ImageCaptureProxy.prototype.takePhoto
        .apply(this.ptr.getProxy(), arguments);
  };

  ImageCaptureProxy.prototype.takePhoto = function(sourceId) {
    var params_ = new ImageCapture_TakePhoto_Params();
    params_.sourceId = sourceId;
    return new Promise(function(resolve, reject) {
      var builder = new codec.MessageV1Builder(
          kImageCapture_TakePhoto_Name,
          codec.align(ImageCapture_TakePhoto_Params.encodedSize),
          codec.kMessageExpectsResponse, 0);
      builder.encodeStruct(ImageCapture_TakePhoto_Params, params_);
      var message = builder.finish();
      this.receiver_.acceptAndExpectResponse(message).then(function(message) {
        var reader = new codec.MessageReader(message);
        var responseParams =
            reader.decodeStruct(ImageCapture_TakePhoto_ResponseParams);
        resolve(responseParams);
      }).catch(function(result) {
        reject(Error("Connection error: " + result));
      });
    }.bind(this));
  };

  function ImageCaptureStub(delegate) {
    this.delegate_ = delegate;
  }
  ImageCaptureStub.prototype.getPhotoState = function(sourceId) {
    return this.delegate_ && this.delegate_.getPhotoState && this.delegate_.getPhotoState(sourceId);
  }
  ImageCaptureStub.prototype.setOptions = function(sourceId, settings) {
    return this.delegate_ && this.delegate_.setOptions && this.delegate_.setOptions(sourceId, settings);
  }
  ImageCaptureStub.prototype.takePhoto = function(sourceId) {
    return this.delegate_ && this.delegate_.takePhoto && this.delegate_.takePhoto(sourceId);
  }

  ImageCaptureStub.prototype.accept = function(message) {
    var reader = new codec.MessageReader(message);
    switch (reader.messageName) {
    default:
      return false;
    }
  };

  ImageCaptureStub.prototype.acceptWithResponder =
      function(message, responder) {
    var reader = new codec.MessageReader(message);
    switch (reader.messageName) {
    case kImageCapture_GetPhotoState_Name:
      var params = reader.decodeStruct(ImageCapture_GetPhotoState_Params);
      this.getPhotoState(params.sourceId).then(function(response) {
        var responseParams =
            new ImageCapture_GetPhotoState_ResponseParams();
        responseParams.state = response.state;
        var builder = new codec.MessageV1Builder(
            kImageCapture_GetPhotoState_Name,
            codec.align(ImageCapture_GetPhotoState_ResponseParams.encodedSize),
            codec.kMessageIsResponse, reader.requestID);
        builder.encodeStruct(ImageCapture_GetPhotoState_ResponseParams,
                             responseParams);
        var message = builder.finish();
        responder.accept(message);
      });
      return true;
    case kImageCapture_SetOptions_Name:
      var params = reader.decodeStruct(ImageCapture_SetOptions_Params);
      this.setOptions(params.sourceId, params.settings).then(function(response) {
        var responseParams =
            new ImageCapture_SetOptions_ResponseParams();
        responseParams.success = response.success;
        var builder = new codec.MessageV1Builder(
            kImageCapture_SetOptions_Name,
            codec.align(ImageCapture_SetOptions_ResponseParams.encodedSize),
            codec.kMessageIsResponse, reader.requestID);
        builder.encodeStruct(ImageCapture_SetOptions_ResponseParams,
                             responseParams);
        var message = builder.finish();
        responder.accept(message);
      });
      return true;
    case kImageCapture_TakePhoto_Name:
      var params = reader.decodeStruct(ImageCapture_TakePhoto_Params);
      this.takePhoto(params.sourceId).then(function(response) {
        var responseParams =
            new ImageCapture_TakePhoto_ResponseParams();
        responseParams.blob = response.blob;
        var builder = new codec.MessageV1Builder(
            kImageCapture_TakePhoto_Name,
            codec.align(ImageCapture_TakePhoto_ResponseParams.encodedSize),
            codec.kMessageIsResponse, reader.requestID);
        builder.encodeStruct(ImageCapture_TakePhoto_ResponseParams,
                             responseParams);
        var message = builder.finish();
        responder.accept(message);
      });
      return true;
    default:
      return false;
    }
  };

  function validateImageCaptureRequest(messageValidator) {
    var message = messageValidator.message;
    var paramsClass = null;
    switch (message.getName()) {
      case kImageCapture_GetPhotoState_Name:
        if (message.expectsResponse())
          paramsClass = ImageCapture_GetPhotoState_Params;
      break;
      case kImageCapture_SetOptions_Name:
        if (message.expectsResponse())
          paramsClass = ImageCapture_SetOptions_Params;
      break;
      case kImageCapture_TakePhoto_Name:
        if (message.expectsResponse())
          paramsClass = ImageCapture_TakePhoto_Params;
      break;
    }
    if (paramsClass === null)
      return validator.validationError.NONE;
    return paramsClass.validate(messageValidator, messageValidator.message.getHeaderNumBytes());
  }

  function validateImageCaptureResponse(messageValidator) {
   var message = messageValidator.message;
   var paramsClass = null;
   switch (message.getName()) {
      case kImageCapture_GetPhotoState_Name:
        if (message.isResponse())
          paramsClass = ImageCapture_GetPhotoState_ResponseParams;
        break;
      case kImageCapture_SetOptions_Name:
        if (message.isResponse())
          paramsClass = ImageCapture_SetOptions_ResponseParams;
        break;
      case kImageCapture_TakePhoto_Name:
        if (message.isResponse())
          paramsClass = ImageCapture_TakePhoto_ResponseParams;
        break;
    }
    if (paramsClass === null)
      return validator.validationError.NONE;
    return paramsClass.validate(messageValidator, messageValidator.message.getHeaderNumBytes());
  }

  var ImageCapture = {
    name: 'media.mojom.ImageCapture',
    kVersion: 0,
    ptrClass: ImageCapturePtr,
    proxyClass: ImageCaptureProxy,
    stubClass: ImageCaptureStub,
    validateRequest: validateImageCaptureRequest,
    validateResponse: validateImageCaptureResponse,
  };
  ImageCaptureStub.prototype.validator = validateImageCaptureRequest;
  ImageCaptureProxy.prototype.validator = validateImageCaptureResponse;
  exports.MeteringMode = MeteringMode;
  exports.RedEyeReduction = RedEyeReduction;
  exports.FillLightMode = FillLightMode;
  exports.Range = Range;
  exports.PhotoState = PhotoState;
  exports.Point2D = Point2D;
  exports.PhotoSettings = PhotoSettings;
  exports.Blob = Blob;
  exports.ImageCapture = ImageCapture;
  exports.ImageCapturePtr = ImageCapturePtr;
  exports.ImageCaptureAssociatedPtr = ImageCaptureAssociatedPtr;
})();