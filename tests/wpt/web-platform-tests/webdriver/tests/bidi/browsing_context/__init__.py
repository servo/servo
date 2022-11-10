def assert_browsing_context(
    info, context, children=None, is_root=True, parent=None, url=None
):
    assert "children" in info
    if children is not None:
        assert isinstance(info["children"], list)
        assert len(info["children"]) == children
    else:
        assert info["children"] is None

    assert "context" in info
    assert isinstance(info["context"], str)
    # Note: Only the tests for browsingContext.getTree should be allowed to
    # pass None here because it's not possible to assert the exact browsing
    # context id for frames.
    if context is not None:
        assert info["context"] == context

    if is_root:
        if parent is None:
            # For a top-level browsing context there is no parent
            assert info["parent"] is None
        else:
            assert "parent" in info
            assert isinstance(info["parent"], str)
            assert info["parent"] == parent
    else:
        # non root browsing context entries do not contain a parent
        assert "parent" not in info
        assert parent is None

    assert "url" in info
    assert isinstance(info["url"], str)
    assert info["url"] == url


def assert_navigation_info(event, context, url):
    assert "context" in event
    assert isinstance(event["context"], str)
    assert event["context"] == context

    assert "url" in event
    assert isinstance(event["url"], str)
    assert event["url"] == url

    assert "navigation" in event
    if event["navigation"] is not None:
        assert isinstance(event["navigation"], str)
