import pytest

from tests.support.inline import inline


# 15.1.3 "Let source be the result returned from the outerHTML IDL attribute
#         of the document element"
def test_source_matches_outer_html(session):
    session.url = inline("<html><head><title>Cheese</title><body>Peas")
    expected_source = session.execute_script(
        "return document.documentElement.outerHTML")

    assert session.source == expected_source

