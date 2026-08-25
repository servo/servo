// META: global=window,dedicatedworker
// META: script=/webcodecs/video-encoder-utils.js
// META: variant=?h264_annexb_software
// META: variant=?h264_annexb_hardware
// META: variant=?h265_annexb_software
// META: variant=?h265_annexb_hardware

var ENCODER_CONFIG = null;
var ANNEXB_CODEC = ''
promise_setup(async () => {
  const config = {
    '?h264_annexb_software': {
      codec: 'avc1.42001E',
      avc: {format: 'annexb'},
      hardwareAcceleration: 'prefer-software',
    },
    '?h264_annexb_hardware': {
      codec: 'avc1.42001E',
      avc: {format: 'annexb'},
      hardwareAcceleration: 'prefer-hardware',
    },
    '?h265_annexb_software': {
      codec: 'hvc1.1.6.L123.00',
      hevc: {format: 'annexb'},
      hardwareAcceleration: 'prefer-software',
    },
    '?h265_annexb_hardware': {
      codec: 'hvc1.1.6.L123.00',
      hevc: {format: 'annexb'},
      hardwareAcceleration: 'prefer-hardware',
    }
  }[location.search];
  if (config.avc) {
    ANNEXB_CODEC = 'h264'
  }
  if (config.hevc) {
    ANNEXB_CODEC = 'h265'
  }
  config.width = 320;
  config.height = 200;
  config.bitrate = 1000000;
  config.framerate = 30;
  ENCODER_CONFIG = config;
});

// The code is inspired from https://source.chromium.org/chromium/chromium/src/+/main:media/formats/mp4/avc.cc;l=190;drc=a6567f4fac823a8a319652bdb5070b5b72a60f30
// and https://source.chromium.org/chromium/chromium/src/+/main:media/formats/mp4/hevc.cc;l=425;drc=a6567f4fac823a8a319652bdb5070b5b72a60f30?

function checkNaluSyntax(test, chunk) {
  test.step(() => {

    const buffer = new Uint8Array(chunk.byteLength);
    const keyFrame = chunk.type === "key";
    chunk.copyTo(buffer);

    const kAUDAllowed = 1;
    const kBeforeFirstVCL = 2;  // VCL == nal_unit_types 1-5
    const kAfterFirstVCL = 3;
    const kEOStreamAllowed = 4;
    const kNoMoreDataAllowed = 5;
    // Define constants for h264 NALU types
    const kAUD = 9;
    const kSEIMessage = 6;
    const kPrefix = 14;
    const kSubsetSPS = 15;
    const kDPS = 16;
    const kReserved17 = 17;
    const kReserved18 = 18;
    const kPPS = 8;
    const kSPS = 7;
    const kSPSExt = 13;
    const kNonIDRSlice = 1;
    const kSliceDataA = 2;
    const kSliceDataB = 3;
    const kSliceDataC = 4;
    const kIDRSlice = 5;
    const kCodedSliceAux = 19;
    const kEOSeq = 10;
    const kEOStream = 11;
    const kFiller = 12;
    const kUnspecified = 0;
    // Define constants for h265 NALU types
    const AUD_NUT = 35;
    const VPS_NUT = 32;
    const SPS_NUT = 33;
    const PPS_NUT = 34;
    const PREFIX_SEI_NUT = 39;
    const RSV_NVCL41 = 41;
    const RSV_NVCL42 = 42;
    const RSV_NVCL43 = 43;
    const RSV_NVCL44 = 44;
    const UNSPEC48 = 48;
    const UNSPEC49 = 49;
    const UNSPEC50 = 50;
    const UNSPEC51 = 51;
    const UNSPEC52 = 52;
    const UNSPEC53 = 53;
    const UNSPEC54 = 54;
    const UNSPEC55 = 55;
    const FD_NUT = 38;
    const SUFFIX_SEI_NUT = 40;
    const RSV_NVCL45 = 45;
    const RSV_NVCL46 = 46;
    const RSV_NVCL47 = 47;
    const UNSPEC56 = 56;
    const UNSPEC57 = 57;
    const UNSPEC58 = 58;
    const UNSPEC59 = 59;
    const UNSPEC60 = 60;
    const UNSPEC61 = 61;
    const UNSPEC62 = 62;
    const UNSPEC63 = 63;
    const EOS_NUT = 36;
    const EOB_NUT = 37;
    const TRAIL_N = 0;
    const TRAIL_R = 1;
    const TSA_N = 2;
    const TSA_R = 3;
    const STSA_N = 4;
    const STSA_R = 5;
    const RADL_N = 6;
    const RADL_R = 7;
    const RASL_N = 8;
    const RASL_R = 9;
    const RSV_VCL_N10 = 10;
    const RSV_VCL_R11 = 11;
    const RSV_VCL_N12 = 12;
    const RSV_VCL_R13 = 13;
    const RSV_VCL_N14 = 14;
    const RSV_VCL_R15 = 15;
    const RSV_VCL24 = 24;
    const RSV_VCL25 = 25;
    const RSV_VCL26 = 26;
    const RSV_VCL27 = 27;
    const RSV_VCL28 = 28;
    const RSV_VCL29 = 29;
    const RSV_VCL30 = 30;
    const RSV_VCL31 = 31;
    const BLA_W_LP = 16;
    const BLA_W_RADL = 17;
    const BLA_N_LP = 18;
    const IDR_W_RADL = 19;
    const IDR_N_LP = 20;
    const CRA_NUT = 21;
    const RSV_IRAP_VCL22 = 22;
    const RSV_IRAP_VCL23 = 23;

    let order_state = kAUDAllowed;
    let lastBytes = [0xFF, 0xFF, 0xFF];
    for (let pos = 0; pos < buffer.length; pos++) {
      if (lastBytes[0] == 0x00 && lastBytes[1] == 0x00
        && lastBytes[2] == 0x01) {
        let naluType = buffer[pos] & 0x1f;
        if (ANNEXB_CODEC === "h264") {
          switch (naluType) {
            case kAUD:
              assert_less_than_equal(order_state, kAUDAllowed, "Unexpected AUD in order_state " + order_state);
              order_state = kBeforeFirstVCL;
              break;

            case kSEIMessage:
            case kPrefix:
            case kSubsetSPS:
            case kDPS:
            case kReserved17:
            case kReserved18:
            case kPPS:
            case kSPS:
              assert_less_than_equal(order_state, kBeforeFirstVCL, "Unexpected NALU type " + naluType + " in order_state " + order_state);
              order_state = kBeforeFirstVCL;
              break;

            case kSPSExt:
              assert_equals(last_nalu_type, kSPS, "SPS extension does not follow an SPS.");
              break;

            case kNonIDRSlice:
            case kSliceDataA:
            case kSliceDataB:
            case kSliceDataC:
            case kIDRSlice:
              assert_less_than_equal(order_state, kAfterFirstVCL, "Unexpected VCL in order_state " + order_state);
              assert_equals(naluType == kIDRSlice, keyFrame, "Keyframe indicator does not match: " + (naluType == kIDRSlice) + " versus " + keyFrame);
              order_state = kAfterFirstVCL;
              break;

            case kCodedSliceAux:
              assert_equals(order_state, kAfterFirstVCL, "Unexpected extension in order_state " + order_state);
              break;

            case kEOSeq:
              assert_equals(order_state, kAfterFirstVCL, "Unexpected EOSeq in order_state " + order_state);
              order_state = kEOStreamAllowed;
              break;

            case kEOStream:
              assert_greater_than(kAfterFirstVCL, order_state, "Unexpected EOStream in order_state " + order_state);
              order_state = kNoMoreDataAllowed;
              break;
            // These syntax elements are to simply be ignored according to H264
            // Annex B 7.4.2.7
            case kFiller:
            case kUnspecified:
              // These syntax elements are to simply be ignored according to H264 Annex B 7.4.2.7
              break;

            default:
              assert_greater_than(naluType, 19, "NALU TYPE smaller than 20 for unknown type");
              break;
          }
        } else if (ANNEXB_CODEC === 'h265') {
          // When any VPS NAL units, SPS NAL units, PPS NAL units, prefix SEI NAL
          // units, NAL units with nal_unit_type in the range of
          // RSV_NVCL41..RSV_NVCL44, or NAL units with nal_unit_type in the range of
          // UNSPEC48..UNSPEC55 are present, they shall not follow the last VCL NAL
          // unit of the access unit.

          switch (naluType) {
            case AUD_NUT:
              assert_less_than_equal(order_state, kAUDAllowed, "Unexpected AUD in order_state " + order_state);
              order_state = kBeforeFirstVCL;
              break;

            case VPS_NUT:
            case SPS_NUT:
            case PPS_NUT:
            case PREFIX_SEI_NUT:
            case RSV_NVCL41:
            case RSV_NVCL42:
            case RSV_NVCL43:
            case RSV_NVCL44:
            case UNSPEC48:
            case UNSPEC49:
            case UNSPEC50:
            case UNSPEC51:
            case UNSPEC52:
            case UNSPEC53:
            case UNSPEC54:
            case UNSPEC55:
              assert_less_than_equal(order_state, kBeforeFirstVCL, "Unexpected NALU type " + nalu.nal_unit_type + " in order_state " + order_state);
              order_state = kBeforeFirstVCL;
              break;
            // NAL units having nal_unit_type equal to FD_NUT or SUFFIX_SEI_NUT or in
            // the range of RSV_NVCL45..RSV_NVCL47 or UNSPEC56..UNSPEC63 shall not
            // precede the first VCL NAL unit of the access unit.
            case FD_NUT:
            case SUFFIX_SEI_NUT:
            case RSV_NVCL45:
            case RSV_NVCL46:
            case RSV_NVCL47:
            case UNSPEC56:
            case UNSPEC57:
            case UNSPEC58:
            case UNSPEC59:
            case UNSPEC60:
            case UNSPEC61:
            case UNSPEC62:
            case UNSPEC63:
              assert_less_than_equal(order_state, kAfterFirstVC, "Unexpected NALU type " + nalu.nal_unit_type + " in order_state " + order_state);
              break;

            // When an end of sequence NAL unit is present, it shall be the last NAL
            // unit among all NAL units in the access unit other than an end of
            // bitstream NAL unit (when present).
            case EOS_NUT:
              assert_equals(order_state, kAfterFirstVCL, "Unexpected EOS in order_state " + order_state);
              order_state = kEOBitstreamAllowed;
              break;
            // When an end of bitstream NAL unit is present, it shall be the last NAL
            // unit in the access unit.
            case EOB_NUT:
              assert_less_than_equal(order_state, kAfterFirstVCL, "Unexpected EOB in order_state " + order_state);
              order_state = kNoMoreDataAllowed;
              break;
            // VCL, non-IRAP
            case TRAIL_N:
            case TRAIL_R:
            case TSA_N:
            case TSA_R:
            case STSA_N:
            case STSA_R:
            case RADL_N:
            case RADL_R:
            case RASL_N:
            case RASL_R:
            case RSV_VCL_N10:
            case RSV_VCL_R11:
            case RSV_VCL_N12:
            case RSV_VCL_R13:
            case RSV_VCL_N14:
            case RSV_VCL_R15:
            case RSV_VCL24:
            case RSV_VCL25:
            case RSV_VCL26:
            case RSV_VCL27:
            case RSV_VCL28:
            case RSV_VCL29:
            case RSV_VCL30:
            case RSV_VCL31:
              assert_less_than_equal(order_state, kAfterFirstVCL, "Unexpected VCL in order_state " + order_state);
              order_state = kAfterFirstVCL;
              break;
            // VCL, IRAP
            case BLA_W_LP:
            case BLA_W_RADL:
            case BLA_N_LP:
            case IDR_W_RADL:
            case IDR_N_LP:
            case CRA_NUT:
            case RSV_IRAP_VCL22:
            case RSV_IRAP_VCL23:
              assert_less_than_equal(order_state, kAfterFirstVCL, "Unexpected VCL in order_state " + order_state);
              assert_equals(keyFrame, true, "The frame is coded as Keyframe, but indicator does not match");
              order_state = kAfterFirstVCL;
              break;

            default:
              assert_true(false, "Unsupported NALU type " + naluType);
              break;
          };

        }
        last_nalu_type = naluType;
      }
      lastBytes.push(buffer[pos]);
      lastBytes.shift(); // advance reading
    }
  })
}

async function runAnnexBTest(t) {
  let encoder_config = { ...ENCODER_CONFIG };
  const w = encoder_config.width;
  const h = encoder_config.height;
  let frames_to_encode = 16;

  await checkEncoderSupport(t, encoder_config);

  const encodedResults = [];
  const encoder_init = {
    output(chunk, metadata) {
      encodedResults.push(chunk);
    },
    error(e) {
      assert_unreached(e.message);
    }
  };

  let encoder = new VideoEncoder(encoder_init);
  encoder.configure(encoder_config);

  for (let i = 0; i < frames_to_encode; i++) {
    let frame = createDottedFrame(w, h, i);
    let keyframe = (i % 5 == 0);
    encoder.encode(frame, { keyFrame: keyframe });
    frame.close();
  }

  await encoder.flush();
  encoder.close();

  encodedResults.forEach((chunk) => checkNaluSyntax(t, chunk));

  assert_greater_than(encodedResults.length, 0, "frames_encoded");
}

promise_test(async t => {
  return runAnnexBTest(t);
}, 'Verify stream compliance h26x annexb');
