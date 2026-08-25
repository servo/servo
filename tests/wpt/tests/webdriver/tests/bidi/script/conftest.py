import pytest
import pytest_asyncio
from typing import Any, List, Mapping, Optional

from webdriver.bidi.modules.script import ContextTarget, OwnershipModel, SerializationOptions


@pytest.fixture
def call_function(bidi_session, top_context):
    async def call_function(
        function_declaration: str,
        arguments: List[Mapping[str, Any]] = [],
        this: Any = None,
        context: str = top_context["context"],
        sandbox: str = None,
        result_ownership: OwnershipModel = OwnershipModel.NONE.value,
        serialization_options: Optional[SerializationOptions] = None,
    ) -> Mapping[str, Any]:
        if sandbox is None:
            target = ContextTarget(context)
        else:
            target = ContextTarget(context, sandbox)

        result = await bidi_session.script.call_function(
            function_declaration=function_declaration,
            arguments=arguments,
            this=this,
            await_promise=False,
            result_ownership=result_ownership,
            serialization_options=serialization_options,
            target=target,
        )
        return result

    return call_function


@pytest_asyncio.fixture
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
        serialization_options: Optional[SerializationOptions] = None,
    ) -> Mapping[str, Any]:
        if sandbox is None:
            target = ContextTarget(context)
        else:
            target = ContextTarget(context, sandbox)

        result = await bidi_session.script.evaluate(
            expression=expression,
            await_promise=False,
            result_ownership=result_ownership,
            serialization_options=serialization_options,
            target=target,
        )
        return result

    return evaluate


@pytest_asyncio.fixture
async def setup_csp_tentative_test(
    bidi_session, inline, top_context, add_preload_script
):
    url = inline(
        """<!DOCTYPE html>
          <html lang="en">
            <head>
              <meta http-equiv="Content-Security-Policy" content="default-src 'unsafe-inline'">
            </head>
            <body onclick="return eval('2 + 1')">
              <script type="text/javascript">
                window.inlineScriptEval = function () {
                  return eval("2 + 1");
                }
              </script>
            </body>
          </html>""",
        # Note: Use the html_quirks template in order to have an empty template
        # and be able to set cleanly a meta tag, a body tag etc. However we are
        # not actually testing quirks mode here, so we still add a standard
        # doctype in the template content.
        doctype="html_quirks",
    )

    await add_preload_script(
        function_declaration="""() => {
          window.preloadScriptEval = function () {
            return eval("2 + 1");
          };
          window.preloadScriptAsyncEval = async function () {
            await new Promise(r => setTimeout(r, 0));
            return eval("2 + 1");
          };
        }""",
        contexts=[top_context["context"]],
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )
