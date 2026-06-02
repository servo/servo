import json

import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def get_navigator_client_hints(bidi_session):
    async def get_navigator_client_hints(context):
        # script to get all low entropy and high entropy values
        expression = """
            (async () => {
                if (!navigator.userAgentData) {
                    return null;
                }
                const highEntropy = await navigator.userAgentData.getHighEntropyValues([
                    "architecture",
                    "bitness",
                    "model",
                    "platformVersion",
                    "fullVersionList",
                    "wow64"
                ]);
                return JSON.stringify({
                    brands: navigator.userAgentData.brands,
                    mobile: navigator.userAgentData.mobile,
                    platform: navigator.userAgentData.platform,
                    ...highEntropy
                });
            })()"""
        result = await bidi_session.script.evaluate(
            expression=expression,
            target=ContextTarget(context["context"]),
            await_promise=True,
        )
        if result["type"] == "null":
            return None
        return json.loads(result["value"])

    return get_navigator_client_hints


@pytest_asyncio.fixture
async def assert_navigation_client_hints(bidi_session, url):
    """
    Helper to assert the right client hints headers sent in navigation request.
    """

    async def assert_navigation_client_hints(context, expected_hints):
        echo_link = url("webdriver/tests/support/http_handlers/headers_echo.py")
        await bidi_session.browsing_context.navigate(context=context["context"],
                                                     url=echo_link,
                                                     wait="complete")

        result = await bidi_session.script.evaluate(
            expression="window.document.body.textContent",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )

        headers = (json.JSONDecoder().decode(result["value"]))["headers"]

        def check_header(header_name, expected_value):
            header_name_lower = header_name.lower()
            if expected_value is not None:
                assert header_name_lower in headers, f"Header {header_name} not found"
                actual_value = headers[header_name_lower][0]
                if isinstance(expected_value, bool):
                    expected_header_val = "?1" if expected_value else "?0"
                    assert actual_value == expected_header_val, f"{header_name} mismatch"
                elif header_name == "Sec-CH-UA" or header_name == "Sec-CH-UA-Full-Version-List":
                    if isinstance(expected_value, str):
                        assert actual_value == expected_value, f"{header_name} mismatch"
                    else:
                        parts = []
                        for item in expected_value:
                            parts.append(
                                f'"{item["brand"]}";v="{item["version"]}"')
                        expected_str = ", ".join(parts)
                        assert actual_value == expected_str, f"{header_name} mismatch"
                else:
                    assert actual_value == expected_value, f"{header_name} mismatch"

        if expected_hints:
            if "brands" in expected_hints:
                check_header("Sec-CH-UA", expected_hints["brands"])
            if "mobile" in expected_hints:
                check_header("Sec-CH-UA-Mobile", expected_hints["mobile"])
            if "platform" in expected_hints:
                check_header("Sec-CH-UA-Platform",
                             f'"{expected_hints["platform"]}"')
            if "platform" in expected_hints:
                check_header("Sec-CH-UA-Platform",
                             f'"{expected_hints["platform"]}"')

    return assert_navigation_client_hints


@pytest_asyncio.fixture
async def assert_client_hints(get_navigator_client_hints,
        assert_navigation_client_hints):
    async def assert_client_hints(context, expected_hints):
        actual_js_hints = await get_navigator_client_hints(context)

        if expected_hints is not None:
            if actual_js_hints is not None:
                for key, value in expected_hints.items():
                    assert actual_js_hints[
                               key] == value, f"JS property {key} mismatch"
            await assert_navigation_client_hints(context, expected_hints)

    return assert_client_hints


@pytest_asyncio.fixture
async def default_client_hints(top_context, get_navigator_client_hints):
    return await get_navigator_client_hints(top_context)
