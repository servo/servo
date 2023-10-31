#!/bin/bash
# This generates the png and jxl images needed.

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
