import pytest
from webdriver.bidi.modules.script import ContextTarget, ScriptEvaluateResultException

from .. import specific_error_response

pytestmark = pytest.mark.asyncio


# The spec doesn't say what happens when serializing the thrown value fails. Reporting
# the value's type without its contents still tells the client what was thrown.
@pytest.mark.parametrize(
    "expression, expected_type",
    [
        ("throw { get foo() { throw new Error('bar'); } }", "object"),
        ("throw (() => { let a = []; for (let i = 1; i < 500; ++i) a = [a]; return a; })()", "array"),
    ],
    ids=["throwing_getter", "deeply_nested"],
)
async def test_unserializable_exception(bidi_session, top_context, expression, expected_type):
    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.evaluate(
            expression=expression,
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )

    specific_error_response({"type": expected_type})(exception.value.result)
