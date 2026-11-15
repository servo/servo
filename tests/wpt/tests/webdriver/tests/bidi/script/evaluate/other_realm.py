import pytest
from webdriver.bidi.modules.script import ContextTarget
from webdriver.bidi.undefined import UNDEFINED

from ... import recursive_compare

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "expression, expected",
    [
        ("new (frames[0].Array)(1, 2)", {
            "type": "array",
            "value": [{"type": "number", "value": 1}, {"type": "number", "value": 2}],
        }),
        ("new (frames[0].Map)([[1, 2]])", {
            "type": "map",
            "value": [[{"type": "number", "value": 1}, {"type": "number", "value": 2}]],
        }),
        ("frames[0].Array.prototype", {"type": "array", "value": []}),
        ("new (frames[0].Set)([1])", {
            "type": "set",
            "value": [{"type": "number", "value": 1}],
        }),
        ("new (frames[0].Date)(0)", {"type": "date", "value": "1970-01-01T00:00:00.000Z"}),
        ("new (frames[0].RegExp)('foo', 'g')", {
            "type": "regexp",
            "value": {"pattern": "foo", "flags": "g"},
        }),
        ("new (frames[0].Error)('foo')", {"type": "error"}),
        ("frames[0].Promise.resolve()", {"type": "promise"}),
        ("frames[0].location", {"type": "object", "value": UNDEFINED}),
        ("frames[0].document.querySelector('div')", {
            "type": "node",
            "value": {
                "attributes": {},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
                "shadowRoot": None,
            },
        }),
        ("frames[0].document.querySelectorAll('div')", {
            "type": "nodelist",
            "value": [{"type": "node"}],
        }),
        ("frames[0].document.getElementsByTagName('div')", {
            "type": "htmlcollection",
            "value": [{"type": "node"}],
        }),
    ],
)
async def test_value_from_iframe_realm(
    bidi_session, top_context, inline, iframe, expression, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(iframe("<div>foo</div>")),
        wait="complete",
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)


async def test_shadow_root_from_iframe_realm(bidi_session, top_context, inline, iframe):
    # The iframe's script never touches the shadow root, so serializing the host from the top-level
    # realm is what first gives the shadow root a JavaScript object. That object has to belong to
    # the iframe's realm.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(iframe("<div><template shadowrootmode=open>foo</template></div>")),
        wait="complete",
    )

    result = await bidi_session.script.evaluate(
        expression="frames[0].document.querySelector('div')",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    recursive_compare({
        "type": "node",
        "value": {"shadowRoot": {"type": "node", "value": {"nodeType": 11, "mode": "open"}}},
    }, result)

    result = await bidi_session.script.evaluate(
        expression="""Object.getPrototypeOf(frames[0].document.querySelector('div').shadowRoot)
            === frames[0].ShadowRoot.prototype""",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    assert result == {"type": "boolean", "value": True}
