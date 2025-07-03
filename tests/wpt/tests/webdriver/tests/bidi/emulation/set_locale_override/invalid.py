import pytest

import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [False, 42, "foo", {}])
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            contexts=value
        )


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            contexts=[])


@pytest.mark.parametrize("value", [None, False, 42, [], {}])
async def test_params_contexts_context_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            contexts=[value])


async def test_params_contexts_entry_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            contexts=["_invalid_"],
        )


async def test_params_contexts_iframe(bidi_session, new_tab, get_test_page):
    url = get_test_page(as_frame=True)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"])
    assert len(contexts) == 1
    frames = contexts[0]["children"]
    assert len(frames) == 1

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            contexts=[frames[0]["context"]],
        )


@pytest.mark.parametrize("value", [True, "foo", 42, {}])
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            user_contexts=value,
        )


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            user_contexts=[],
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            user_contexts=[value],
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_user_contexts_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.emulation.set_locale_override(
            locale=None,
            user_contexts=[value],
        )


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_locale_invalid_type(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=value,
            contexts=[top_context["context"]],
        )


@pytest.mark.parametrize("value", [
    # Is an empty string.
    "",
    # Language subtag is too short.
    "a",
    # Language subtag is too long.
    "abcd",
    # Language subtag contains invalid characters (numbers).
    "12",
    # Language subtag contains invalid characters (symbols).
    "en$",
    # Uses underscore instead of hyphen as a separator.
    "en_US",
    # Region subtag is too short.
    "en-U",
    # Region subtag is too long.
    "en-USAXASDASD",
    # Region subtag contains invalid characters (numbers not part of a valid UN M49 code).
    "en-U1",
    # Region subtag contains invalid characters (symbols).
    "en-US$",
    # Script subtag is too short.
    "en-Lat",
    # Script subtag is too long.
    "en-Somelongsubtag",
    # Script subtag contains invalid characters (numbers).
    "en-La1n",
    # Script subtag contains invalid characters (symbols).
    "en-Lat$",
    # Variant subtag is too short (must be 5-8 alphanumeric chars, or 4 if starting with a digit).
    "en-US-var",
    # Variant subtag contains invalid characters (symbols).
    "en-US-variant$",
    # Extension subtag is malformed (singleton "u" not followed by anything).
    "en-u-",
    # Extension subtag is malformed (singleton "t" not followed by anything).
    "de-t-",
    # Private use subtag "x-" is not followed by anything.
    "x-",
    # Locale consisting only of a private use subtag.
    "x-another-private-tag",
    # Private use subtag contains invalid characters (underscore).
    "en-x-private_use",
    # Contains an empty subtag (double hyphen).
    "en--US",
    # Starts with a hyphen.
    "-en-US",
    # Ends with a hyphen.
    "en-US-",
    # Contains only a hyphen.
    "-",
    # Contains non-ASCII characters.
    "en-US-ñ",
    # Grandfathered tag with invalid structure.
    "i-notarealtag",
    # Invalid UN M49 region code (not 3 digits).
    "en-01",
    # Invalid UN M49 region code (contains letters).
    "en-0A1",
    # Malformed language tag with numbers.
    "123",
    # Locale with only script.
    "Latn",
    # Locale with script before language.
    "Latn-en",
    # Repeated separator.
    "en--US",
    # Invalid character in an otherwise valid structure.
    "en-US-!",
    # Too many subtags of a specific type (e.g., multiple script tags).
    "en-Latn-Cyrl-US"
])
async def test_params_locale_invalid_value(bidi_session, top_context, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.emulation.set_locale_override(
            locale=value,
            contexts=[top_context["context"]],
        )
