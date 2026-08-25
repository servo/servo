#include <jxl/encode.h>
#include <jxl/thread_parallel_runner.h>
#include <stdio.h>
#include <stdlib.h>

/* Creates an animated JXL with 4 solid-color 8x8 frames at 500ms each.
   Usage: make_anim_jxl <num_loops> <output.jxl>
   num_loops: 0=infinite, 1=once, 2=twice, etc.
*/

static const uint8_t kColors[4][3] = {
    {255, 0, 0},   /* red    */
    {0, 255, 0},   /* green  */
    {0, 0, 255},   /* blue   */
    {255, 255, 0}, /* yellow */
};

int main(int argc, char** argv) {
  if (argc != 3) {
    fprintf(stderr, "usage: %s <num_loops> <output.jxl>\n", argv[0]);
    return 1;
  }
  uint32_t num_loops = (uint32_t)atoi(argv[1]);
  const char* outpath = argv[2];

  const size_t W = 8, H = 8;

  JxlEncoder* enc = JxlEncoderCreate(NULL);
  if (!enc) {
    fprintf(stderr, "JxlEncoderCreate failed\n");
    return 1;
  }

  void* runner = JxlThreadParallelRunnerCreate(NULL, 1);
  JxlEncoderSetParallelRunner(enc, JxlThreadParallelRunner, runner);

  JxlBasicInfo info;
  JxlEncoderInitBasicInfo(&info);
  info.xsize = W;
  info.ysize = H;
  info.bits_per_sample = 8;
  info.num_color_channels = 3;
  info.num_extra_channels = 0;
  info.alpha_bits = 0;
  info.have_animation = JXL_TRUE;
  info.animation.tps_numerator = 2; /* 2 ticks per second */
  info.animation.tps_denominator = 1;
  info.animation.num_loops = num_loops;
  info.animation.have_timecodes = JXL_FALSE;
  info.uses_original_profile = JXL_TRUE; /* required for lossless */

  if (JxlEncoderSetBasicInfo(enc, &info) != JXL_ENC_SUCCESS) {
    fprintf(stderr, "JxlEncoderSetBasicInfo failed\n");
    return 1;
  }

  JxlColorEncoding color;
  JxlColorEncodingSetToSRGB(&color, JXL_FALSE);
  if (JxlEncoderSetColorEncoding(enc, &color) != JXL_ENC_SUCCESS) {
    fprintf(stderr, "JxlEncoderSetColorEncoding failed\n");
    return 1;
  }

  JxlEncoderFrameSettings* settings = JxlEncoderFrameSettingsCreate(enc, NULL);
  JxlEncoderSetFrameLossless(settings, JXL_TRUE);

  JxlPixelFormat fmt = {3, JXL_TYPE_UINT8, JXL_NATIVE_ENDIAN, 0};

  for (int f = 0; f < 4; f++) {
    uint8_t pixels[W * H * 3];
    for (size_t i = 0; i < W * H; i++) {
      pixels[i * 3 + 0] = kColors[f][0];
      pixels[i * 3 + 1] = kColors[f][1];
      pixels[i * 3 + 2] = kColors[f][2];
    }

    JxlFrameHeader fh;
    JxlEncoderInitFrameHeader(&fh);
    fh.duration = 1; /* 1 tick at 2 ticks/sec = 500ms */
    if (JxlEncoderSetFrameHeader(settings, &fh) != JXL_ENC_SUCCESS) {
      fprintf(stderr, "JxlEncoderSetFrameHeader failed\n");
      return 1;
    }
    if (JxlEncoderAddImageFrame(settings, &fmt, pixels, sizeof(pixels)) !=
        JXL_ENC_SUCCESS) {
      fprintf(stderr, "JxlEncoderAddImageFrame failed\n");
      return 1;
    }
  }

  JxlEncoderCloseInput(enc);

  /* Collect output */
  size_t buf_size = 4096;
  uint8_t* buf = malloc(buf_size);
  size_t total = 0;
  JxlEncoderStatus st;
  do {
    uint8_t* p = buf + total;
    size_t avail = buf_size - total;
    st = JxlEncoderProcessOutput(enc, &p, &avail);
    total = buf_size - avail;
    if (st == JXL_ENC_NEED_MORE_OUTPUT) {
      buf_size *= 2;
      buf = realloc(buf, buf_size);
    }
  } while (st == JXL_ENC_NEED_MORE_OUTPUT);

  if (st != JXL_ENC_SUCCESS) {
    fprintf(stderr, "JxlEncoderProcessOutput failed\n");
    return 1;
  }

  FILE* fp = fopen(outpath, "wb");
  fwrite(buf, 1, total, fp);
  fclose(fp);

  free(buf);
  JxlEncoderDestroy(enc);
  JxlThreadParallelRunnerDestroy(runner);

  printf("wrote %zu bytes to %s\n", total, outpath);
  return 0;
}
