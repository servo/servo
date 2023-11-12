# https://github.com/http2jp/http2-frame-test-case

import os
import json
import pytest
from hyperframe.frame import Frame

tc_filepaths = []
root = os.path.dirname(__file__)
path = os.walk(os.path.join(root, "http2-frame-test-case"))
for dirpath, dirnames, filenames in path:
    for filename in filenames:
        if os.path.splitext(filename)[1] != ".json":
            continue
        tc_filepaths.append(
            os.path.relpath(os.path.join(dirpath, filename), root)
        )


def check_valid_frame(tc, data):  # noqa: C901
    new_frame, length = Frame.parse_frame_header(data[:9], strict=True)
    new_frame.parse_body(memoryview(data[9:9 + length]))

    assert tc["frame"]["length"] == length
    assert tc["frame"]["stream_identifier"] == new_frame.stream_id
    assert tc["frame"]["type"] == new_frame.type

    flags = 0
    for flag, flag_bit in new_frame.defined_flags:
        if flag in new_frame.flags:
            flags |= flag_bit
    assert tc["frame"]["flags"] == flags

    p = tc["frame"]["frame_payload"]
    if "header_block_fragment" in p:
        assert p["header_block_fragment"] == new_frame.data.decode()
    if "data" in p:
        assert p["data"] == new_frame.data.decode()
    if "padding" in p:
        # the padding data itself is not retained by hyperframe after parsing
        pass
    if "padding_length" in p and p["padding_length"]:
        assert p["padding_length"] == new_frame.pad_length
    if "error_code" in p:
        assert p["error_code"] == new_frame.error_code
    if "additional_debug_data" in p:
        assert p["additional_debug_data"].encode() == new_frame.additional_data
    if "last_stream_id" in p:
        assert p["last_stream_id"] == new_frame.last_stream_id
    if "stream_dependency" in p:
        assert p["stream_dependency"] or 0 == new_frame.depends_on
    if "weight" in p and p["weight"]:
        assert p["weight"] - 1 == new_frame.stream_weight
    if "exclusive" in p:
        assert (p["exclusive"] or False) == new_frame.exclusive
    if "opaque_data" in p:
        assert p["opaque_data"].encode() == new_frame.opaque_data
    if "promised_stream_id" in p:
        assert p["promised_stream_id"] == new_frame.promised_stream_id
    if "settings" in p:
        assert dict(p["settings"]) == new_frame.settings
    if "window_size_increment" in p:
        assert p["window_size_increment"] == new_frame.window_increment


class TestExternalCollection:
    @pytest.mark.parametrize('tc_filepath', tc_filepaths)
    def test(self, tc_filepath):
        with open(os.path.join(root, tc_filepath)) as f:
            tc = json.load(f)

        data = bytes.fromhex(tc["wire"])

        if tc["error"] is None and tc["frame"]:
            check_valid_frame(tc, data)
        elif tc["error"] and tc["frame"] is None:
            with pytest.raises(Exception):
                new_frame, length = Frame.parse_frame_header(
                    data[:9],
                    strict=True
                )
                new_frame.parse_body(memoryview(data[9:9 + length]))
                assert length == new_frame.body_len
        else:
            pytest.fail("unexpected test case: {} {}".format(tc_filepath, tc))
