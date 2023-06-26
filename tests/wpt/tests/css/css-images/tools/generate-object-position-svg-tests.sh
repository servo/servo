#!/bin/bash
#
# Any copyright is dedicated to the Public Domain.
# http://creativecommons.org/publicdomain/zero/1.0/
#
# This is a script that I used to generate a suite of tests for the CSS
# properties "object-fit" and "object-position" (focusing on edge-case
# object-position values that require pixel rounding), using a template
# testcase file and reference case file.
#
# The reference case uses the "background-size" & "background-position"
# equivalent of the tested "object-fit" / "object-position" values.

FILE_PATH="./"
REFTEST_LIST_FILE="$FILE_PATH/reftest.list"

TEMPLATE_TESTCASE_FILENAME=$FILE_PATH/support/template-object-position-test.html
TEMPLATE_REFERENCE_FILENAME=$FILE_PATH/support/template-object-position-ref.html

imageFileFormat="svg"

# Array of image files to use for testing:
imageFileArr=("support/colors-16x8.svg" "support/colors-8x16.svg")
numImageFiles=${#imageFileArr[@]}

# Array of CSS classes to delete from the template, for a given image-file.
# DETAILS: The template files contain some elements/styles that exercise
# object-position x-values (op_x), and other elements/styles that exercise
# object-position y-values (op_y). But actually, we'll only have extra space
# for these percent values to resolve against in *one* dimension (since our
# image-files are rectangular, and the container element is square, and we
# scale the image up with "object-fit: contain"). So, we delete the
# elements/styles in the dimension where object-position % values will just
# resolve to 0 ("op_x" for the fat image, and "op_y" for the tall image).
classPatternToDeleteArr=("op_x" "op_y")

# Array of tag-names for elements that we'd like to test:
# (Also: array of a single-letter abbreviation for each element, an array of
# the close tag for each element -- if a close tag is needed -- and an array
# indicating the attribute that each element uses to specify its image source.)
tagNameArr=(       "embed" "img" "object"    "video" )
tagLetterArr=(     "e"     "i"   "o"         "p" )
tagCloseTokenArr=( ""      ""    "</object>" "</video>" )
tagSrcAttrArr=(        "src"   "src" "data"      "poster" )
numTags=${#tagNameArr[@]}

  for ((j = 0; j < $numImageFiles; j++)); do
    imageFile=${imageFileArr[$j]}

    classPatternToDelete=${classPatternToDeleteArr[$j]}

    let testNum=$j+1
    testNum="00$testNum" # zero-pad to 3 digits, per w3c convention

    filenameStub="object-position-$imageFileFormat-$testNum"

    # Generate a reference case:
    filenameRef="$filenameStub-ref.html"
    echo Generating ${filenameRef}.
    cat $TEMPLATE_REFERENCE_FILENAME \
      | sed "s,REPLACEME_IMAGE_FILENAME,$imageFile," \
      | sed "/$classPatternToDelete/d" \
      > $FILE_PATH/$filenameRef

    # Generate a test for each of our tags:
    for ((k = 0; k < $numTags; k++)); do
      tagName=${tagNameArr[$k]}
      tagLetter=${tagLetterArr[$k]}
      tagCloseToken=${tagCloseTokenArr[$k]}
      tagSrcAttr=${tagSrcAttrArr[$k]}

      filenameTest="$filenameStub$tagLetter.html"
      testTitle="various 'object-position' values on a fixed-size $tagName element, with a SVG image and 'object-fit:contain'."
      echo Generating ${filenameTest}.
      cat $TEMPLATE_TESTCASE_FILENAME \
        | sed "s,REPLACEME_IMAGE_FILENAME,$imageFile," \
        | sed "s/REPLACEME_TEST_TITLE/$testTitle/" \
        | sed "s,REPLACEME_REFERENCE_FILENAME,$filenameRef," \
        | sed "s/REPLACEME_CONTAINER_TAG/$tagName/" \
        | sed "s,REPLACEME_CONTAINER_CLOSETAG,$tagCloseToken,"  \
        | sed "s/REPLACEME_SRC_ATTR/$tagSrcAttr/" \
        | sed "/$classPatternToDelete/d" \
        > $FILE_PATH/$filenameTest

      echo "== $filenameTest $filenameRef" \
        >> $REFTEST_LIST_FILE
    done
  done
