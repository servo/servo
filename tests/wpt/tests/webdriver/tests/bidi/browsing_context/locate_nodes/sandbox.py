import pytest

from webdriver.bidi.modules.script import ContextTarget,OwnershipModel


@pytest.mark.asyncio
async def test_locate_nodes_in_sandbox(bidi_session, inline, top_context):
    url = inline("""<div data-class="one">foobarBARbaz</div><div data-class="two">foobarBARbaz</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "div[data-class='one']" },
        sandbox="sandbox"
    )

    assert len(result["nodes"]) == 1
    node_id = result["nodes"][0]["sharedId"]

    # Since the node was found in the sandbox, it should be available
    # to scripts running in the sandbox.
    result_in_sandbox = await bidi_session.script.call_function(
        function_declaration="function(){ return arguments[0]; }",
        target=ContextTarget(top_context["context"], "sandbox"),
        await_promise=True,
        arguments=[
            {
                "sharedId": node_id
            }
        ]
    )
    assert result_in_sandbox["type"] == "node"
    assert result_in_sandbox["sharedId"] == node_id


@pytest.mark.asyncio
async def test_locate_same_node_in_different_sandboxes_returns_same_id(bidi_session, inline, top_context):
    url = inline("""<div data-class="one">foobarBARbaz</div><div data-class="two">foobarBARbaz</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    first_result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "div[data-class='one']" },
        sandbox="first_sandbox"
    )

    assert len(first_result["nodes"]) == 1

    second_result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "div[data-class='one']" },
        sandbox="second_sandbox"
    )
    assert len(second_result["nodes"]) == 1
    assert first_result["nodes"][0]["sharedId"] == second_result["nodes"][0]["sharedId"]


@pytest.mark.asyncio
async def test_locate_same_node_in_default_sandbox_returns_same_id_as_sandbox(bidi_session, inline, top_context):
    url = inline("""<div data-class="one">foobarBARbaz</div><div data-class="two">foobarBARbaz</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "div[data-class='one']" }
    )

    assert len(result["nodes"]) == 1
    node_id = result["nodes"][0]["sharedId"]

    result_in_sandbox = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "div[data-class='one']" },
        sandbox="sandbox"
    )
    assert len(result_in_sandbox["nodes"]) == 1
    assert result_in_sandbox["nodes"][0]["sharedId"] == node_id


@pytest.mark.asyncio
async def test_locate_same_node_in_different_sandboxes_with_root_ownership_returns_different_handles(bidi_session, inline, top_context):
    url = inline("""<div data-class="one">foobarBARbaz</div><div data-class="two">foobarBARbaz</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    first_result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "div[data-class='one']" },
        ownership=OwnershipModel.ROOT.value,
        sandbox="first_sandbox"
    )

    assert len(first_result["nodes"]) == 1

    second_result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "div[data-class='one']" },
        ownership=OwnershipModel.ROOT.value,
        sandbox="second_sandbox"
    )

    assert len(second_result["nodes"]) == 1
    assert first_result["nodes"][0]["sharedId"] == second_result["nodes"][0]["sharedId"]
    assert first_result["nodes"][0]["handle"] != second_result["nodes"][0]["handle"]
