#!/bin/bash
set -eu

# This generates the png and jxl images needed for JPEG XL WPT tests.
#
# Part 1: generates core 3x3 fixtures from scratch (requires convert, cjxl, djxl).
# Part 2: copies additional fixtures from a local jxl-rs checkout (optional).
#
# Optional jxl-rs fixtures are sourced from:
#   ${JXL_RS_TESTDATA:-$HOME/jxl-rs/jxl/resources/test}

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
JXL_RS_CLI="${JXL_RS_CLI:-$HOME/jxl-rs/target/release/jxl_cli}"

# Function to check if a command exists
command_exists() {
  command -v "$1" &> /dev/null
}

# Function to convert and compress an image
convert_and_compress() {
  local input_image=$1
  local base_name=$2
  local color_space=$3

  cjxl "$input_image" temp.jxl -d 0
  djxl temp.jxl "${base_name}_${color_space}_lossless.png"
  cjxl "${base_name}_${color_space}_lossless.png" "${base_name}_${color_space}_lossy.jxl" -d 0.0001 -e 7
  djxl "${base_name}_${color_space}_lossy.jxl" "${base_name}_${color_space}_lossy.png"
  cjxl "${base_name}_${color_space}_lossless.png" "${base_name}_${color_space}_lossless.jxl" -d 0
}

copy_if_exists() {
  local src="$1"
  local dst="$2"
  if [[ -f "$src" ]]; then
    cp "$src" "$dst"
    echo "copied: $dst"
  else
    echo "missing: $src"
  fi
}

decode_png_ref_if_exists() {
  local src_jxl="$1"
  local dst_png="$2"
  if [[ -f "$src_jxl" ]]; then
    if [[ "$(basename "$src_jxl")" == "conformance_sunset_logo.jxl" && -x "$JXL_RS_CLI" ]]; then
      # Keep this PNG aligned with Chromium's jxl-rs decoder output.
      "$JXL_RS_CLI" --override-bitdepth 8 "$src_jxl" "$dst_png" >/dev/null 2>&1
      echo "generated with jxl_cli: $dst_png"
    elif command -v djxl >/dev/null 2>&1; then
      djxl "$src_jxl" "$dst_png" >/dev/null 2>&1
      echo "generated: $dst_png"
    else
      echo "missing tools: djxl or $JXL_RS_CLI (cannot generate $dst_png)"
    fi
  else
    echo "missing source for png generation: $src_jxl"
  fi
}

# --- Part 1: Core 3x3 fixtures ---

# Check for required tools
for tool in convert cjxl djxl; do
  if ! command_exists "$tool"; then
    echo "$tool could not be found. Please install it and run the script again."
    exit 1
  fi
done

# Create a 3x3 transparent image
convert -size 3x3 xc:none 3x3a.png

# Draw colors with alpha values
convert 3x3a.png \
-fill "rgba(255,0,0,0.5)" -draw "point 0,0" \
-fill "rgba(0,255,0,0.5)" -draw "point 1,0" \
-fill "rgba(0,0,255,0.5)" -draw "point 2,0" \
-fill "rgba(128,64,64,0.5)" -draw "point 0,1" \
-fill "rgba(64,128,64,0.5)" -draw "point 1,1" \
-fill "rgba(64,64,128,0.5)" -draw "point 2,1" \
-fill "rgba(255,255,255,0.5)" -draw "point 0,2" \
-fill "rgba(128,128,128,0.5)" -draw "point 1,2" \
-fill "rgba(0,0,0,0.5)" -draw "point 2,2" \
3x3a.png

# Generate initial image with alpha values
generate_image 3x3a.png

# Generate a version without alpha channel
convert 3x3a.png -alpha off 3x3.png

# Define color spaces
# TODO(firsching): add "RGB_D65_202_Rel_PeQ" and "RGB_D65_202_Rel_HLG" as color spaces here
color_spaces=("srgb")

# Loop through color spaces and convert/compress images
for color_space in "${color_spaces[@]}"; do
  convert_and_compress 3x3.png "3x3" "$color_space"
  convert_and_compress 3x3a.png "3x3a" "$color_space"
done

convert 3x3.png -quality 70 3x3.jpg
# lossless recompression
cjxl 3x3.jpg 3x3_jpeg_recompression.jxl
# checking that it was actually byte exact
djxl 3x3_jpeg_recompression.jxl 3x3_recovered.jpg
diff 3x3.jpg 3x3_recovered.jpg
if [ $? -ne 0 ]; then
  echo "The recovery of the recompressed jpg failed: 3x3.png and 3x3_recovered.jpg differ"
  exit 1
fi
# generate reference png
djxl 3x3_jpeg_recompression.jxl 3x3_jpeg_recompression.png

# Cleanup temporary file
rm -f temp.jxl 3x3.png 3x3a.png 3x3.jpg 3x3_recovered.jpg

# --- Part 2: Optional jxl-rs fixtures ---

JXL_RS_TESTDATA="${JXL_RS_TESTDATA:-$HOME/jxl-rs/jxl/resources/test}"
JXL_RS_CONF="$JXL_RS_TESTDATA/conformance_test_images"

for f in \
  basic.jxl \
  8x8_noise.jxl \
  with_icc.jxl \
  orientation1_identity.jxl \
  orientation6_rotate_90_cw.jxl \
  orientation8_rotate_90_ccw.jxl \
  green_queen_modular_e3.jxl \
  green_queen_vardct_e3.jxl \
  has_permutation.jxl \
  progressive_ac.jxl \
  spline_on_first_frame.jxl \
  issue648_palette0.jxl \
  hdr_pq_test.jxl \
  hdr_hlg_test.jxl
  do
  copy_if_exists "$JXL_RS_TESTDATA/$f" "$SCRIPT_DIR/$f"
done

copy_if_exists "$JXL_RS_CONF/alpha_nonpremultiplied.jxl" \
  "$SCRIPT_DIR/conformance_alpha_nonpremultiplied.jxl"
copy_if_exists "$JXL_RS_CONF/sunset_logo.jxl" \
  "$SCRIPT_DIR/conformance_sunset_logo.jxl"
copy_if_exists "$JXL_RS_CONF/cmyk_layers.jxl" \
  "$SCRIPT_DIR/conformance_cmyk_layers.jxl"
copy_if_exists "$JXL_RS_CONF/animation_spline.jxl" \
  "$SCRIPT_DIR/conformance_animation_spline.jxl"

# PNG references used by reftests.
decode_png_ref_if_exists \
  "$SCRIPT_DIR/green_queen_modular_e3.jxl" \
  "$SCRIPT_DIR/green_queen_modular_e3.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/conformance_cmyk_layers.jxl" \
  "$SCRIPT_DIR/conformance_cmyk_layers.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/orientation6_rotate_90_cw.jxl" \
  "$SCRIPT_DIR/orientation6_rotate_90_cw.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/conformance_sunset_logo.jxl" \
  "$SCRIPT_DIR/conformance_sunset_logo.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/conformance_alpha_nonpremultiplied.jxl" \
  "$SCRIPT_DIR/conformance_alpha_nonpremultiplied.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/has_permutation.jxl" \
  "$SCRIPT_DIR/has_permutation.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/basic.jxl" \
  "$SCRIPT_DIR/basic.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/orientation1_identity.jxl" \
  "$SCRIPT_DIR/orientation1_identity.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/orientation8_rotate_90_ccw.jxl" \
  "$SCRIPT_DIR/orientation8_rotate_90_ccw.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/with_icc.jxl" \
  "$SCRIPT_DIR/with_icc.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/issue648_palette0.jxl" \
  "$SCRIPT_DIR/issue648_palette0.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/spline_on_first_frame.jxl" \
  "$SCRIPT_DIR/spline_on_first_frame.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/progressive_ac.jxl" \
  "$SCRIPT_DIR/progressive_ac.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/green_queen_vardct_e3.jxl" \
  "$SCRIPT_DIR/green_queen_vardct_e3.png"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/8x8_noise.jxl" \
  "$SCRIPT_DIR/8x8_noise.png"

# --- Part 3: Hand-crafted edge case bitstreams ---

# Smallest valid naked JXL codestream (12 bytes, 512x256 black).
# See crbug.com/484214291.
printf '\xff\x0a\xff\x07\x08\x83\x04\x0c\x00\x4b\x20\x18' \
  > "$SCRIPT_DIR/smallest_valid.jxl"
echo "generated: $SCRIPT_DIR/smallest_valid.jxl"
decode_png_ref_if_exists \
  "$SCRIPT_DIR/smallest_valid.jxl" \
  "$SCRIPT_DIR/smallest_valid.png"

echo "Done: $SCRIPT_DIR"
