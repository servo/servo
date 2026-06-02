import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget

from ... import remote_mapping_to_dict


@pytest_asyncio.fixture
async def get_current_screen_dimensions(bidi_session):
    async def get_current_screen_dimensions(context):
        result = await bidi_session.script.evaluate(
            expression="""({
                "width": screen.width,
                "height": screen.height,
                "availWidth": screen.availWidth,
                "availHeight": screen.availHeight,
            })""",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )

        return remote_mapping_to_dict(result["value"])

    return get_current_screen_dimensions


@pytest_asyncio.fixture
async def match_min_device_width_media_query(match_media_query):
    async def match_min_device_width_media_query(context, media_query_value):
        return await match_media_query(context, "min-device-width", media_query_value)

    return match_min_device_width_media_query


@pytest_asyncio.fixture
async def assert_screen_dimensions(
    get_current_screen_dimensions,
    match_min_device_width_media_query,
):
    async def assert_screen_dimensions(
        context,
        screen_width,
        screen_height,
        screen_avail_width,
        screen_avail_height,
    ):
        assert await get_current_screen_dimensions(context) == {
            "width": screen_width,
            "height": screen_height,
            "availWidth": screen_avail_width,
            "availHeight": screen_avail_height,
        }

        assert (
            await match_min_device_width_media_query(context, screen_width - 1) is True
        )
        assert (
            await match_min_device_width_media_query(context, screen_width + 1) is False
        )

    return assert_screen_dimensions
