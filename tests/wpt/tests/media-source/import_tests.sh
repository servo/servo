#!/bin/bash

if [ $# -lt 1 ]
then
  echo "Usage: $0 <Blink directory>"
  exit -1
fi

BLINK_ROOT=$1
LAYOUT_TEST_DIR=$BLINK_ROOT/LayoutTests
HTTP_MEDIA_TEST_DIR=$LAYOUT_TEST_DIR/http/tests/media

if [ ! -d "$BLINK_ROOT" ]
then
  echo "$BLINK_ROOT is not a directory or doesn't exist"
  exit -1
fi

if [ ! -d "$LAYOUT_TEST_DIR" ]
then
  echo "$LAYOUT_TEST_DIR is not a directory or doesn't exist"
  exit -1
fi

#rm -rf *.html *.js webm mp4 manifest.txt

cp $HTTP_MEDIA_TEST_DIR/media-source/mediasource-*.html $HTTP_MEDIA_TEST_DIR/media-source/mediasource-*.js .
cp -r $HTTP_MEDIA_TEST_DIR/resources/media-source/webm .
cp -r $HTTP_MEDIA_TEST_DIR/resources/media-source/mp4 .

# Remove Blink-specific files
rm mediasource-gc-after-decode-error-crash.html

sed -i 's/\/w3c\/resources\//\/resources\//g' *.html
sed -i 's/\/media\/resources\/media-source\///g' *.html
sed -i 's/\/media\/resources\/media-source\///g' *.js
sed -i 's/\/media\/resources\/media-source\///g' webm/*


for TEST_FILE in `ls *.html`
do
  if [ "$TEST_FILE" = "index.html" ]
  then
    continue
  fi
  echo -e "$TEST_FILE" >> manifest.txt
done

cp import_tests-template.txt index.html

chmod -R a+r *.html *.js webm mp4 manifest.txt
chmod a+rx webm mp4
