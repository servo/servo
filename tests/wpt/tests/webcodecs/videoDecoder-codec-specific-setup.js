const AV1_DATA = {
  src: 'av1.mp4',
  config: {
    codec: 'av01.0.04M.08',
    codedWidth: 320,
    codedHeight: 240,
    visibleRect: {x: 0, y: 0, width: 320, height: 240},
    displayWidth: 320,
    displayHeight: 240,
  },
  chunks: [
    {offset: 48, size: 1938}, {offset: 1986, size: 848},
    {offset: 2834, size: 3}, {offset: 2837, size: 47}, {offset: 2884, size: 3},
    {offset: 2887, size: 116}, {offset: 3003, size: 3},
    {offset: 3006, size: 51}, {offset: 3057, size: 25},
    {offset: 3082, size: 105}
  ]
};

const VP8_DATA = {
  src: 'vp8.webm',
  config: {
    codec: 'vp8',
    codedWidth: 320,
    codedHeight: 240,
    visibleRect: {x: 0, y: 0, width: 320, height: 240},
    displayWidth: 320,
    displayHeight: 240,
  },
  chunks: [
    {offset: 522, size: 4826}, {offset: 5355, size: 394},
    {offset: 5756, size: 621}, {offset: 6384, size: 424},
    {offset: 6815, size: 532}, {offset: 7354, size: 655},
    {offset: 8016, size: 670}, {offset: 8693, size: 2413},
    {offset: 11113, size: 402}, {offset: 11522, size: 686}
  ]
};

const VP9_DATA = {
  src: 'vp9.mp4',
  // TODO(sandersd): Verify that the file is actually level 1.
  config: {
    codec: 'vp09.00.10.08',
    codedWidth: 320,
    codedHeight: 240,
    displayAspectWidth: 320,
    displayAspectHeight: 240,
  },
  chunks: [
    {offset: 44, size: 3315}, {offset: 3359, size: 203},
    {offset: 3562, size: 245}, {offset: 3807, size: 172},
    {offset: 3979, size: 312}, {offset: 4291, size: 170},
    {offset: 4461, size: 195}, {offset: 4656, size: 181},
    {offset: 4837, size: 356}, {offset: 5193, size: 159}
  ]
};

const H264_AVC_DATA = {
  src: 'h264.mp4',
  config: {
    codec: 'avc1.64000b',
    description: {offset: 9490, size: 45},
    codedWidth: 320,
    codedHeight: 240,
    displayAspectWidth: 320,
    displayAspectHeight: 240,
  },
  chunks: [
    {offset: 48, size: 4140}, {offset: 4188, size: 604},
    {offset: 4792, size: 475}, {offset: 5267, size: 561},
    {offset: 5828, size: 587}, {offset: 6415, size: 519},
    {offset: 6934, size: 532}, {offset: 7466, size: 523},
    {offset: 7989, size: 454}, {offset: 8443, size: 528}
  ]
};

const H264_ANNEXB_DATA = {
  src: 'h264.annexb',
  config: {
    codec: 'avc1.64000b',
    codedWidth: 320,
    codedHeight: 240,
    displayAspectWidth: 320,
    displayAspectHeight: 240,
  },
  chunks: [
    {offset: 0, size: 4175}, {offset: 4175, size: 602},
    {offset: 4777, size: 473}, {offset: 5250, size: 559},
    {offset: 5809, size: 585}, {offset: 6394, size: 517},
    {offset: 6911, size: 530}, {offset: 7441, size: 521},
    {offset: 7962, size: 452}, {offset: 8414, size: 526}
  ]
};

const H265_HEVC_DATA = {
  src: 'h265.mp4',
  config: {
    codec: 'hev1.1.6.L60.90',
    description: {offset: 5821, size: 2406},
    codedWidth: 320,
    codedHeight: 240,
    displayAspectWidth: 320,
    displayAspectHeight: 240,
  },
  chunks: [
    {offset: 44, size: 2515}, {offset: 2559, size: 279},
    {offset: 2838, size: 327}, {offset: 3165, size: 329},
    {offset: 3494, size: 308}, {offset: 3802, size: 292},
    {offset: 4094, size: 352}, {offset: 4446, size: 296},
    {offset: 4742, size: 216}, {offset: 4958, size: 344}
  ]
};

const H265_ANNEXB_DATA = {
  src: 'h265.annexb',
  config: {
    codec: 'hev1.1.6.L60.90',
    codedWidth: 320,
    codedHeight: 240,
    displayAspectWidth: 320,
    displayAspectHeight: 240,
  },
  chunks: [
    {offset: 0, size: 4894}, {offset: 4894, size: 279},
    {offset: 5173, size: 327}, {offset: 5500, size: 329},
    {offset: 5829, size: 308}, {offset: 6137, size: 292},
    {offset: 6429, size: 352}, {offset: 6781, size: 296},
    {offset: 7077, size: 216}, {offset: 7293, size: 344}
  ]
};

// Allows mutating `callbacks` after constructing the VideoDecoder, wraps calls
// in t.step().
function createVideoDecoder(t, callbacks) {
  return new VideoDecoder({
    output(frame) {
      if (callbacks && callbacks.output) {
        t.step(() => callbacks.output(frame));
      } else {
        t.unreached_func('unexpected output()');
      }
    },
    error(e) {
      if (callbacks && callbacks.error) {
        t.step(() => callbacks.error(e));
      } else {
        t.unreached_func('unexpected error()');
      }
    }
  });
}

function createCorruptChunk(index) {
  let bad_data = CHUNK_DATA[index];
  for (var i = 0; i < bad_data.byteLength; i += 4)
    bad_data[i] = 0xFF;
  return new EncodedVideoChunk(
      {type: 'delta', timestamp: index, data: bad_data});
}

// Create a view of an ArrayBuffer.
function view(buffer, {offset, size}) {
  return new Uint8Array(buffer, offset, size);
}

async function checkImplements() {
  // Don't run any tests if the codec is not supported.
  assert_equals('function', typeof VideoDecoder.isConfigSupported);
  let supported = false;
  try {
    // TODO(sandersd): To properly support H.264 in AVC format, this should
    // include the `description`. For now this test assumes that H.264 Annex B
    // support is the same as H.264 AVC support.
    const support = await VideoDecoder.isConfigSupported({codec: CONFIG.codec});
    supported = support.supported;
  } catch (e) {
  }
  assert_implements_optional(supported, CONFIG.codec + ' unsupported');
}

let CONFIG = null;
let CHUNK_DATA = null;
let CHUNKS = null;
promise_setup(async () => {
  const data = {
    '?av1': AV1_DATA,
    '?vp8': VP8_DATA,
    '?vp9': VP9_DATA,
    '?h264_avc': H264_AVC_DATA,
    '?h264_annexb': H264_ANNEXB_DATA,
    '?h265_hevc': H265_HEVC_DATA,
    '?h265_annexb': H265_ANNEXB_DATA
  }[location.search];

  // Fetch the media data and prepare buffers.
  const response = await fetch(data.src);
  const buf = await response.arrayBuffer();

  CONFIG = {...data.config};
  if (data.config.description) {
    CONFIG.description = view(buf, data.config.description);
  }

  CHUNK_DATA = data.chunks.map((chunk, i) => view(buf, chunk));

  CHUNKS = CHUNK_DATA.map(
      (data, i) => new EncodedVideoChunk(
          {type: i == 0 ? 'key' : 'delta', timestamp: i, duration: 1, data}));
});
