#!/bin/bash
#
# Any copyright is dedicated to the Public Domain.
# http://creativecommons.org/publicdomain/zero/1.0/
#
# Script to generate <canvas src> reftest files for "object-fit" and
# "object-position", from corresponding reftest files that use <object>.
#
# This script expects to be run from the parent directory.

# Array of image files that we'll use
imageFileArr=("support/colors-16x8.png" "support/colors-8x16.png")
canvasAttributeArr=('width="16" height="8"'   'width="8" height="16"')
numImageFiles=${#imageFileArr[@]}


for ((i = 0; i < $numImageFiles; i++)); do

  imageFile=${imageFileArr[$i]}
  canvasAttrs=${canvasAttributeArr[$i]}

  # Loop across <object> tests:
  # (We assume that tests that end with "001" use the first PNG image in
  # $imageFileArr, and tests that end with "002" use the second PNG image.)
  let testNum=$i+1
  for origTestName in object-*-png-*00${testNum}o.html; do
    # Find the corresponding reference case:
    origReferenceName=$(echo $origTestName |
                        sed "s/o.html/-ref.html/")

    # Replace "o" suffix in filename with "c" (for "canvas")
    canvasTestName=$(echo $origTestName |
                     sed "s/o.html/c.html/")

    # Generate testcase
    # (converting <object data="..."> to <canvas width="..." height="...">
    echo "Generating $canvasTestName from $origTestName."
    hg cp $origTestName $canvasTestName

    # Do string-replacements in testcase to convert it to test canvas:
    # Adjust html & body nodes:
    sed -i "s|<html>|<html class=\"reftest-wait\">|" $canvasTestName
    sed -i "s|<body>|<body onload=\"drawImageToCanvases('$imageFile')\">|" $canvasTestName
    # Adjust <title>:
    sed -i "s|object element|canvas element|g" $canvasTestName
    # Tweak the actual tags (open & close tags, and CSS rule):
    sed -i "s|object {|canvas {|" $canvasTestName
    sed -i "s|<object|<canvas|" $canvasTestName
    sed -i "s|</object>|</canvas>|" $canvasTestName
    # Drop "data" attr (pointing to image URI) and replace with
    # width/height attrs to establish the canvas's intrinsic size:
    sed -i "s|data=\"$imageFile\"|$canvasAttrs|" $canvasTestName

    # Add a <script> block to draw an image into each canvas:
    sed -i "/<\/style>/a \\
    <script>\n\
      function drawImageToCanvases(imageURI) {\n\
        var image = new Image();\n\
        image.onload = function() {\n\
          var canvasElems = document.getElementsByTagName(\"canvas\");\n\
          for (var i = 0; i < canvasElems.length; i++) {\n\
            var ctx = canvasElems[i].getContext(\"2d\");\n\
            ctx.drawImage(image, 0, 0);\n\
          }\n\
          document.documentElement.removeAttribute(\"class\");\n\
        }\n\
        image.src = imageURI;\n\
      }\n\
    <\/script>" $canvasTestName
  done
done
