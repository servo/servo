# -*- coding: utf-8 -*-
import pytest
import os
import json
import sys

from hypothesis.strategies import text

if sys.version_info[0] == 2:
    from codecs import open

# We need to grab one text example from hypothesis to prime its cache.
text().example()

# This pair of generator expressions are pretty lame, but building lists is a
# bad idea as I plan to have a substantial number of tests here.
story_directories = (
    os.path.join('test/test_fixtures', d)
    for d in os.listdir('test/test_fixtures')
)
story_files = (
    os.path.join(storydir, name)
    for storydir in story_directories
    for name in os.listdir(storydir)
    if 'raw-data' not in storydir
)
raw_story_files = (
    os.path.join('test/test_fixtures/raw-data', name)
    for name in os.listdir('test/test_fixtures/raw-data')
)


@pytest.fixture(scope='class', params=story_files)
def story(request):
    """
    Provides a detailed HPACK story to test with.
    """
    with open(request.param, 'r', encoding='utf-8') as f:
        return json.load(f)


@pytest.fixture(scope='class', params=raw_story_files)
def raw_story(request):
    """
    Provides a detailed HPACK story to test the encoder with.
    """
    with open(request.param, 'r', encoding='utf-8') as f:
        return json.load(f)
