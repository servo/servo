import pytest
from webdriver.bidi.modules.network import NetworkStringValue
from webdriver.bidi.modules.storage import PartialCookie, BrowsingContextPartitionDescriptor

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "protocol",
    [
        "http",
        "https",
    ],
)
async def test_set_cookie_protocol(bidi_session, top_context, inline, origin, domain_value, protocol):
    # Navigate to a page with a required protocol.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=(inline("<div>foo</div>", protocol=protocol)), wait="complete"
    )

    source_origin = origin(protocol)
    partition = BrowsingContextPartitionDescriptor(top_context["context"])

    set_cookie_result = await bidi_session.storage.set_cookie(
        cookie=PartialCookie(
            name='foo',
            value=NetworkStringValue('bar'),
            domain=domain_value(),
            secure=True
        ),
        partition=partition)

    assert set_cookie_result == {
        'partitionKey': {
            'sourceOrigin': source_origin
        },
    }

    # Assert the cookie is actually set.
    actual_cookies = await bidi_session.storage.get_cookies(partition=partition)
    assert actual_cookies == {
        'cookies': [
            {
                'domain': domain_value(),
                'httpOnly': False,
                'name': 'foo',
                'path': '/',
                'sameSite': 'none',
                'secure': True,
                'size': 6,
                'value': {
                    'type': 'string',
                    'value': 'bar',
                },
            },
        ],
        'partitionKey': {
            'sourceOrigin': source_origin,
        },
    }
