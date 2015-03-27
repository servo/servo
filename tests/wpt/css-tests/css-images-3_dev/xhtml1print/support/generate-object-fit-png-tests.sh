#!/bin/bash
#
# Any copyright is dedicated to the Public Domain.
# http://creativecommons.org/publicdomain/zero/1.0/
#
# This is a script that I used to generate a suite of tests for the CSS
# properties "object-fit" and "object-position", using a template testcase
# file and reference case file.
#
# The reference case uses the "background-size" & "background-position"
# equivalent of the tested "object-fit" / "object-position" values.
# (One exception: since there is no "background-size" equivalent of
# "object-fit: scale-down", we add an extra CSS rule for the "scale-down"
# reference cases -- see REPLACEME_SCALE_DOWN_EXTRA_RULE below.)

FILE_PATH="./"
REFTEST_LIST_FILE="$FILE_PATH/reftest.list"

TEMPLATE_TESTCASE_FILENAME=$FILE_PATH/support/template-object-fit-test.html
TEMPLATE_REFERENCE_FILENAME=$FILE_PATH/support/template-object-fit-ref.html

imageFileFormat="png"

# Array of image files to use for testing:
imageFileArr=("support/colors-16x8.png" "support/colors-8x16.png")
numImageFiles=${#imageFileArr[@]}

# Array of "object-fit" values, & of corresponding "background-size" values.
# (Note: background-size has no equivalent for "object-fit: scale-down". We use
# "auto auto" here, which is equivalent *some* of the time; and for the cases
# where it's *not* equivalent, we use an extra CSS rule in the reference case.
# See REPLACEME_SCALE_DOWN_EXTRA_RULE below.)
objectFitArr=(           "fill"      "contain" "cover" "none"      "scale-down" )
backgroundSizeEquivArr=( "100% 100%" "contain" "cover" "auto auto" "auto auto"  )
numObjectFitVals=${#objectFitArr[@]}

# Array of tag-names for elements that we'd like to test:
# (Also: array of a single-letter abbreviation for each element, an array of
# the close tag for each element -- if a close tag is needed -- and an array
# indicating the attribute that each element uses to specify its image source.)
tagNameArr=(       "embed" "img" "object"    "video" )
tagLetterArr=(     "e"     "i"   "o"         "p" )
tagCloseTokenArr=( ""      ""    "</object>" "</video>" )
tagSrcAttrArr=(        "src"   "src" "data"      "poster" )
numTags=${#tagNameArr[@]}

# FIRST: Add 'default-preferences' line to the top of our reftest manifest:
echo "default-preferences test-pref(layout.css.object-fit-and-position.enabled,true)
# Tests for 'object-fit' / 'object-position' with a PNG image" \
 >> $REFTEST_LIST_FILE

for ((i = 0; i < $numObjectFitVals; i++)); do
  objectFit=${objectFitArr[$i]}
  backgroundSizeEquiv=${backgroundSizeEquivArr[$i]}

  # The reference case for "scale-down" needs an additional style rule, to
  # look like "object-fit: scale-down" on small objects. (This is necessary
  # because "background-size" doesn't have a value that exactly maps to
  # "object-fit: scale-down".)
  if [[ $objectFit == "scale-down" ]]; then
      scaledownRule=".objectOuter > .small { background-size: contain; }"
      scaledownSedCmd="s/REPLACEME_SCALE_DOWN_EXTRA_RULE/$scaledownRule/"
  else
      # (We're not testing "scale-down" in this generated file, so just delete
      # the template's placeholder line for a "scale-down"-specific CSS rule.)
      scaledownSedCmd="/REPLACEME_SCALE_DOWN_EXTRA_RULE/d"
  fi

  for ((j = 0; j < $numImageFiles; j++)); do
    imageFile=${imageFileArr[$j]}
    let testNum=$j+1
    testNum="00$testNum" # zero-pad to 3 digits, per w3c convention

    filenameStub="object-fit-$objectFit-$imageFileFormat-$testNum"

    # Generate a reference case:
    filenameRef="$filenameStub-ref.html"
    echo Generating ${filenameRef}.
    cat $TEMPLATE_REFERENCE_FILENAME \
      | sed "s,REPLACEME_IMAGE_FILENAME,$imageFile," \
      | sed "s/REPLACEME_BACKGROUND_SIZE_VAL/$backgroundSizeEquiv/" \
      | sed "$scaledownSedCmd" \
      > $FILE_PATH/$filenameRef;

    # Generate a test for each of our tags:
    for ((k = 0; k < $numTags; k++)); do
      tagName=${tagNameArr[$k]}
      tagLetter=${tagLetterArr[$k]}
      tagCloseToken=${tagCloseTokenArr[$k]}
      tagSrcAttr=${tagSrcAttrArr[$k]}

      filenameTest="$filenameStub$tagLetter.html"
      testTitle="'object-fit: $objectFit' on $tagName element, with a PNG image and with various 'object-position' values"
      echo Generating ${filenameTest}.
      cat $TEMPLATE_TESTCASE_FILENAME \
        | sed "s,REPLACEME_IMAGE_FILENAME,$imageFile," \
        | sed "s/REPLACEME_TEST_TITLE/$testTitle/" \
        | sed "s,REPLACEME_REFERENCE_FILENAME,$filenameRef," \
        | sed "s/REPLACEME_CONTAINER_TAG/$tagName/" \
        | sed "s,REPLACEME_CONTAINER_CLOSETAG,$tagCloseToken,"  \
        | sed "s/REPLACEME_SRC_ATTR/$tagSrcAttr/" \
        | sed "s/REPLACEME_OBJECT_FIT_VAL/$objectFit/" \
        > $FILE_PATH/$filenameTest

      echo "== $filenameTest $filenameRef" \
        >> $REFTEST_LIST_FILE
    done
  done
done
