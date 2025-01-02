'use strict';

// This file introduces constants used to mock depth data for depth sensing API.

const convertDepthBufferToArrayBuffer = function (data, desiredFormat) {
  if(desiredFormat == "luminance-alpha") {
    const result = new ArrayBuffer(data.length * 2);  // each entry has 2 bytes
    const view = new Uint16Array(result);

    for(let i = 0; i < data.length; ++i) {
      view[i] = data[i];
    }

    return new Uint8Array(result);
  } else if(desiredFormat == "float32") {
    const result = new ArrayBuffer(data.length * 4);  // each entry has 4 bytes
    const view = new Float32Array(result);

    for(let i = 0; i < data.length; ++i) {
      view[i] = data[i];
    }

    return new Uint8Array(result);
  } else {
    throw new Error("Unrecognized data format!");
  }
}

// Let's assume that the depth values are in cm, Xcm = x * 1/100m
const RAW_VALUE_TO_METERS = 1/100;

const createDepthSensingData = function() {
  const depthSensingBufferHeight = 5;
  const depthSensingBufferWidth = 7;
  const depthSensingBuffer = [
    1,  1,  1,   1,   1,    1,    1,  // first row
    1,  2,  3,   4,   5,    6,    7,
    1,  4,  9,  16,  25,   36,   49,
    1,  8, 27,  64, 125,  216,  343,
    1, 16, 81, 256, 625, 1296, 2401,
  ];  // depthSensingBuffer value at column c, row r is Math.pow(c+1, r).

  // Let's assume that the origin of the depth buffer is in the bottom right
  // corner, with X's growing to the left and Y's growing upwards.
  // This corresponds to the origin at 2401 in the above matrix, with X axis
  // growing from 2401 towards 1296, and Y axis growing from 2401 towards 343.
  // This corresponds to a rotation around Z axis by 180 degrees, with origin at [1,1].
  const depthSensingBufferFromViewerTransform = {
    position: [1, 1, 0],
    orientation: [0, 0, 1, 0],
  };

  return {
    depthData: convertDepthBufferToArrayBuffer(depthSensingBuffer, "luminance-alpha"),
    width: depthSensingBufferWidth,
    height: depthSensingBufferHeight,
    normDepthBufferFromNormView: depthSensingBufferFromViewerTransform,
    rawValueToMeters: RAW_VALUE_TO_METERS,
  };
};

const DEPTH_SENSING_DATA = createDepthSensingData();

// Returns expected depth value at |column|, |row| coordinates, expressed
// in depth buffer's coordinate system.
const getExpectedValueAt = function(column, row) {
  return Math.pow(column+1, row) * RAW_VALUE_TO_METERS;
};

const DEPTH_CONFIG_ALL_FORMATS = ['luminance-alpha', 'float32', 'unsigned-short'];
const DEPTH_CONFIG_ALL_USAGES= ['gpu-optimized', 'cpu-optimized'];

const VALID_DEPTH_CONFIG_CPU_USAGE = {
  usagePreference: ['cpu-optimized'],
  dataFormatPreference: ['luminance-alpha', 'float32', 'unsigned-short'],
};

const VALID_DEPTH_CONFIG_GPU_USAGE = {
  usagePreference: ['gpu-optimized'],
  dataFormatPreference: ['luminance-alpha', 'float32', 'unsigned-short'],
};
