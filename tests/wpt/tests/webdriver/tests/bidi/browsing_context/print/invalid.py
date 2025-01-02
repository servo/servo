import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("context", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(context=context)


async def test_params_context_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.print(context="_invalid_")


async def test_params_context_closed(bidi_session):
    new_tab = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.close(context=new_tab["context"])

    # Try to print the closed context
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.print(context=new_tab["context"])


@pytest.mark.parametrize("background", ["foo", 42, {}, []])
async def test_params_background_invalid_type(bidi_session, top_context, background):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], background=background
        )


@pytest.mark.parametrize(
    "margin",
    [
        False,
        "foo",
        42,
        [],
        {"top": False},
        {"top": "foo"},
        {"top": []},
        {"top": {}},
        {"bottom": False},
        {"bottom": "foo"},
        {"bottom": []},
        {"bottom": {}},
        {"left": False},
        {"left": "foo"},
        {"left": []},
        {"left": {}},
        {"right": False},
        {"right": "foo"},
        {"right": []},
        {"right": {}},
    ],
)
async def test_params_margin_invalid_type(bidi_session, top_context, margin):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], margin=margin
        )


@pytest.mark.parametrize(
    "margin",
    [
        {"top": -0.1},
        {"bottom": -0.1},
        {"left": -0.1},
        {"right": -0.1},
    ],
)
async def test_params_margin_invalid_value(bidi_session, top_context, margin):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], margin=margin
        )


@pytest.mark.parametrize("orientation", [False, 42, {}, []])
async def test_params_orientation_invalid_type(bidi_session, top_context, orientation):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], orientation=orientation
        )


async def test_params_orientation_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], orientation="foo"
        )


@pytest.mark.parametrize(
    "page",
    [
        False,
        "foo",
        42,
        [],
        {"height": False},
        {"height": "foo"},
        {"height": []},
        {"height": {}},
        {"width": False},
        {"width": "foo"},
        {"width": []},
        {"width": {}},
    ],
)
async def test_params_page_invalid_type(bidi_session, top_context, page):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], page=page
        )


@pytest.mark.parametrize(
    "page",
    [
        {"height": -1},
        {"width": -1},
        {"height": 0.03},
        {"width": 0.03},
    ],
)
async def test_params_page_invalid_value(bidi_session, top_context, page):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], page=page
        )


@pytest.mark.parametrize(
    "page_ranges",
    [
        False,
        "foo",
        42,
        {},
        [None],
        [False],
        [[]],
        [{}],
        ["1-2", {}],
    ],
)
async def test_params_page_ranges_invalid_type(bidi_session, top_context, page_ranges):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], page_ranges=page_ranges
        )


@pytest.mark.parametrize(
    "page_ranges",
    [
        [4.2],
        ["4.2"],
        ["3-2"],
        ["a-2"],
        ["1:2"],
        ["1-2-3"],
    ],
)
async def test_params_page_ranges_invalid_value(bidi_session, top_context, page_ranges):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], page_ranges=page_ranges
        )


@pytest.mark.parametrize("scale", [False, "foo", {}, []])
async def test_params_scale_invalid_type(bidi_session, top_context, scale):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], scale=scale
        )


@pytest.mark.parametrize("scale", [-1, 0.09, 2.01, 42])
async def test_params_scale_invalid_value(bidi_session, top_context, scale):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], scale=scale
        )


@pytest.mark.parametrize("shrink_to_fit", ["foo", 42, {}, []])
async def test_params_shrink_to_fit_invalid_type(
    bidi_session, top_context, shrink_to_fit
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.print(
            context=top_context["context"], shrink_to_fit=shrink_to_fit
        )
