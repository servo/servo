import os
import glob
import shutil
from os import path


TEST_FILE_PATTERN = "support/**.test"
TEST_OUTPUT_PATH = "tests"

TEMPLATE = """\
<!doctype html>
<!-- DO NOT EDIT! This file and %vtt_file_rel_path are generated. -->
<!-- See /webvtt/parsing/file-parsing/README.md -->
<meta charset=utf-8>
<title>WebVTT parser test: %test_name</title>
%test_headers
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id=log></div>
<script>
var t = async_test('%test_name');
t.step(function(){
    var video = document.createElement('video');
    var track = document.createElement('track');
    assert_true('src' in track, 'track element not supported');
    track.src = '%vtt_file_rel_path';
    track['default'] = true;
    track.kind = 'subtitles';
    track.onload = this.step_func(trackLoaded);
    track.onerror = this.step_func(trackError);
    video.appendChild(track);
    document.body.appendChild(video);
});

function trackLoaded(event) {
    var track = event.target;
    var video = track.parentNode;
    var cues = video.textTracks[0].cues;
    {
%test_js
    }
    this.done();
}

function trackError(e) {
    assert_unreached('got unexpected error event');
}
</script>
"""

def generate_test(test_path, output_dir):
    # Read test file
    test_filename = path.basename(test_path)
    test_basefilename = path.splitext(test_filename)[0]

    with open(test_path, 'r') as test:
        test_source = test.read()

    # Split test header
    splits = test_source.split('\n\n', 1)
    if len(splits) != 2:
        raise ValueError("Leave an empty line between the test header and body")

    test_header, test_body = splits

    # Split header into name + html headers
    splits = test_header.split('\n', 1)

    test_name = splits[0]
    if len(splits) == 2:
        test_headers = splits[1]

    # Split body into js + vtt
    splits = test_body.split('\n===\n', 1)
    if len(splits) != 2:
        raise ValueError("Use === to separate the js and vtt parts")

    test_js, test_vtt = splits

    # Get output paths
    os.makedirs(output_dir, exist_ok=True)
    html_file_path = path.join(output_dir, test_basefilename + '.html')

    vtt_file_dir = path.join(output_dir, 'support')
    os.makedirs(vtt_file_dir, exist_ok=True)

    vtt_file_name = test_basefilename + '.vtt'
    vtt_file_path = path.join(vtt_file_dir, vtt_file_name)
    vtt_file_rel_path = path.join('support', vtt_file_name)

    # Write html file
    with open(html_file_path, 'w') as output:
        html = (TEMPLATE.replace('%test_name', test_name)
                        .replace('%test_headers', test_headers)
                        .replace('%test_js', test_js)
                        .replace('%vtt_file_rel_path', vtt_file_rel_path))
        output.write(html)

    # Write vtt file
    with open(vtt_file_path, 'w') as output:
        encoded = bytes(test_vtt, "utf-8").decode("unicode_escape")
        output.write(encoded)

def main():
    file_parsing_path = path.normpath(path.join(path.dirname(__file__), ".."))

    test_output_path = path.join(file_parsing_path, TEST_OUTPUT_PATH)

    tests_pattern = path.join(file_parsing_path, TEST_FILE_PATTERN)

    # Clean test directory
    shutil.rmtree(test_output_path)

    # Generate tests
    for file in glob.glob(tests_pattern):
        print('Building test files for: ' + file)
        generate_test(file, test_output_path)

if __name__ == '__main__':
    main()
