import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, "foo", False, 42, {}])
async def test_params_data_types_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=value, max_encoded_data_size=1000
        )


async def test_params_data_types_empty_array(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=[], max_encoded_data_size=1000
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_data_types_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=[value], max_encoded_data_size=1000
        )


@pytest.mark.parametrize("value", ["foo", "request", "invalid"])
async def test_params_data_types_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=[value], max_encoded_data_size=1000
        )


@pytest.mark.parametrize("value", [None, "foo", False, {}, []])
async def test_params_max_encoded_data_size_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=value
        )


@pytest.mark.parametrize("value", [0, -1, -100])
async def test_params_max_encoded_data_size_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=value
        )


async def test_params_max_encoded_data_size_exceeds_max_total_size(bidi_session):
    # Use a very large value that should exceed the maximum total size
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=999999999999
        )


# collectorType parameter tests
@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_collector_type_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=1000, collector_type=value
        )


@pytest.mark.parametrize("value", ["foo", "invalid", "stream"])
async def test_params_collector_type_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=1000, collector_type=value
        )


@pytest.mark.parametrize("value", [False, 42, {}, ""])
async def test_params_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=1000, contexts=value
        )


async def test_params_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=1000, contexts=[]
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=1000, contexts=[value]
        )


async def test_params_contexts_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.network.add_data_collector(
            data_types=["response"],
            max_encoded_data_size=1000,
            contexts=["does not exist"],
        )


@pytest.mark.parametrize("value", [False, 42, {}, ""])
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=1000, user_contexts=value
        )


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=1000, user_contexts=[]
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_user_contexts_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"], max_encoded_data_size=1000, user_contexts=[value]
        )


async def test_params_user_contexts_invalid_value(bidi_session):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.network.add_data_collector(
            data_types=["response"],
            max_encoded_data_size=1000,
            user_contexts=["does not exist"],
        )


async def test_params_contexts_and_user_contexts_mutually_exclusive(
    bidi_session, new_tab
):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_data_collector(
            data_types=["response"],
            max_encoded_data_size=1000,
            contexts=[new_tab["context"]],
            user_contexts=["default"],
        )
