#!/bin/sh

: ${JAVA:=java}

# Temporary shell script to help shake out the bugs to webgl2.js until
# it's folded back into the Closure workspace.
$JAVA -jar ../../../../closure/compiler.jar \
  --use_only_custom_externs \
  --js test-webgl2.js \
  --externs temp_externs/chrome.js \
  --externs temp_externs/deprecated.js \
  --externs temp_externs/es3.js \
  --externs temp_externs/es5.js \
  --externs temp_externs/es6.js \
  --externs temp_externs/es6_collections.js \
  --externs temp_externs/fileapi.js \
  --externs temp_externs/flash.js \
  --externs temp_externs/gecko_css.js \
  --externs temp_externs/gecko_dom.js \
  --externs temp_externs/gecko_event.js \
  --externs temp_externs/gecko_xml.js \
  --externs temp_externs/google.js \
  --externs temp_externs/html5.js \
  --externs temp_externs/ie_css.js \
  --externs temp_externs/ie_dom.js \
  --externs temp_externs/ie_event.js \
  --externs temp_externs/ie_vml.js \
  --externs temp_externs/intl.js \
  --externs temp_externs/iphone.js \
  --externs temp_externs/mediasource.js \
  --externs temp_externs/page_visibility.js \
  --externs temp_externs/v8.js \
  --externs temp_externs/w3c_anim_timing.js \
  --externs temp_externs/w3c_css.js \
  --externs temp_externs/w3c_css3d.js \
  --externs temp_externs/w3c_device_sensor_event.js \
  --externs temp_externs/w3c_dom1.js \
  --externs temp_externs/w3c_dom2.js \
  --externs temp_externs/w3c_dom3.js \
  --externs temp_externs/w3c_elementtraversal.js \
  --externs temp_externs/w3c_encoding.js \
  --externs temp_externs/w3c_event.js \
  --externs temp_externs/w3c_event3.js \
  --externs temp_externs/w3c_geolocation.js \
  --externs temp_externs/w3c_indexeddb.js \
  --externs temp_externs/w3c_navigation_timing.js \
  --externs temp_externs/w3c_range.js \
  --externs temp_externs/w3c_rtc.js \
  --externs temp_externs/w3c_selectors.js \
  --externs temp_externs/w3c_xml.js \
  --externs temp_externs/webkit_css.js \
  --externs temp_externs/webkit_dom.js \
  --externs temp_externs/webkit_event.js \
  --externs temp_externs/webkit_notifications.js \
  --externs temp_externs/webstorage.js \
  --externs temp_externs/window.js \
  --externs webgl2.js \
  --compilation_level ADVANCED \
  --warning_level VERBOSE \
  --js_output_file /dev/null
