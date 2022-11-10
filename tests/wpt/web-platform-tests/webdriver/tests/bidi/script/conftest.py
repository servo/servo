import pytest
from typing import Any, List, Mapping

from webdriver.bidi.modules.script import ContextTarget, OwnershipModel


@pytest.fixture
def call_function(bidi_session, top_context):
    async def call_function(
        function_declaration: str,
        arguments: List[Mapping[str, Any]] = [],
        this: Any = None,
        context: str = top_context["context"],
        sandbox: str = None,
        result_ownership: OwnershipModel = OwnershipModel.NONE.value,
    ) -> Mapping[str, Any]:
        if sandbox is None:
            target = ContextTarget(top_context["context"])
        else:
            target = ContextTarget(top_context["context"], sandbox)

        result = await bidi_session.script.call_function(
            function_declaration=function_declaration,
            arguments=arguments,
            this=this,
            await_promise=False,
            result_ownership=result_ownership,
            target=target,
        )
        return result

    return call_function


@pytest.fixture
async def default_realm(bidi_session, top_context):
    realms = await bidi_session.script.get_realms(context=top_context["context"])
    return realms[0]["realm"]


@pytest.fixture
def evaluate(bidi_session, top_context):
    async def evaluate(
        expression: str,
        context: str = top_context["context"],
        sandbox: str = None,
        result_ownership: OwnershipModel = OwnershipModel.NONE.value,
    ) -> Mapping[str, Any]:
        if sandbox is None:
            target = ContextTarget(top_context["context"])
        else:
            target = ContextTarget(top_context["context"], sandbox)

        result = await bidi_session.script.evaluate(
            expression=expression,
            await_promise=False,
            result_ownership=result_ownership,
            target=target,
        )
        return result

    return evaluate
