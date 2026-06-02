import pytest

import webdriver.bidi.error as error
from webdriver.bidi.modules.emulation import CoordinatesOptions
from webdriver.bidi.undefined import UNDEFINED


pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [False, 42, "foo", {}])
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=value,
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
        )


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[],
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
        )


@pytest.mark.parametrize("value", [None, False, 42, [], {}])
async def test_params_contexts_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[value],
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
        )


async def test_params_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=["_invalid_"],
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
        )


async def test_params_contexts_iframe(bidi_session, new_tab, get_test_page):
    url = get_test_page(as_frame=True)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    assert len(contexts) == 1
    frames = contexts[0]["children"]
    assert len(frames) == 1

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[frames[0]["context"]],
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
        )


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_coordinates_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=value,
        )


async def test_params_coordinates_empty_object(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates={},
        )


async def test_params_coordinates_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=UNDEFINED,
        )


async def test_params_coordinates_latitude_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates={
                "longitude": 10,
            },
        )


@pytest.mark.parametrize("value", [None, False, "foo", [], {}])
async def test_params_coordinates_latitude_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=value,
                longitude=10,
            ),
        )


@pytest.mark.parametrize("value", [-90.1, 90.1])
async def test_params_coordinates_latitude_invalid_value(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=value,
                longitude=10,
            ),
        )


async def test_params_coordinates_longitude_missing(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates={
                "latitude": 10,
            },
        )


@pytest.mark.parametrize("value", [None, False, "foo", [], {}])
async def test_params_coordinates_longitude_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=value,
            ),
        )


@pytest.mark.parametrize("value", [-180.5, 180.5])
async def test_params_coordinates_longitude_invalid_value(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=value,
            ),
        )


@pytest.mark.parametrize("value", [False, "foo", [], {}])
async def test_params_coordinates_accuracy_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                accuracy=value,
            ),
        )


async def test_params_coordinates_accuracy_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                accuracy=-1,
            ),
        )


@pytest.mark.parametrize("value", [False, "foo", [], {}])
async def test_params_coordinates_altitude_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                altitude=value,
            ),
        )


@pytest.mark.parametrize("value", [False, "foo", [], {}])
async def test_params_coordinates_altitude_accuracy_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                altitude=10,
                altitude_accuracy=value,
            ),
        )


async def test_params_coordinates_altitude_accuracy_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                altitude=10,
                altitude_accuracy=-1,
            ),
        )


async def test_params_coordinates_altitude_accuracy_without_altitude(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                altitude_accuracy=10,
            ),
        )


@pytest.mark.parametrize("value", [False, "foo", [], {}])
async def test_params_coordinates_heading_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                heading=value,
            ),
        )


@pytest.mark.parametrize("value", [-0.5, 360, 360.5])
async def test_params_coordinates_heading_invalid_value(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                heading=value,
            ),
        )


@pytest.mark.parametrize("value", [False, "foo", [], {}])
async def test_params_coordinates_speed_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                speed=value,
            ),
        )


async def test_params_coordinates_speed_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates=CoordinatesOptions(
                latitude=10,
                longitude=10,
                speed=-1.5,
            ),
        )


@pytest.mark.parametrize("value", [True, "foo", 42, {}])
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
            user_contexts=value,
        )


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
            user_contexts=[],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
            user_contexts=[value],
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_user_contexts_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.emulation.set_geolocation_override(
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
            user_contexts=[value],
        )


async def test_params_coordinates_and_error(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            coordinates={
                "latitude": 10,
                "longitude": 10,
            },
            error={"type": "positionUnavailable"}
        )


async def test_params_no_coordinates_no_error(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_error_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            error=value,
        )


async def test_params_error_empty_object(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            error={},
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_error_type_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            error={
                "type": value
            },
        )


async def test_params_error_type_invalid_value(bidi_session, top_context):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_geolocation_override(
            contexts=[top_context["context"]],
            error={
                "type": "unknownError",
            },
        )
