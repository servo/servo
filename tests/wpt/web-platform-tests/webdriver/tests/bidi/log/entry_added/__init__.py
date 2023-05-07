from webdriver.bidi.modules.script import ContextTarget

from ... import any_int, any_list, any_string, create_console_api_message, recursive_compare


def assert_base_entry(
    entry,
    level=any_string,
    text=any_string,
    timestamp=any_int,
    realm=any_string,
    context=None,
    stacktrace=None
):
    recursive_compare({
        "level": level,
        "text": text,
        "timestamp": timestamp,
        "source": {
            "realm": realm
        }
    }, entry)

    if stacktrace is not None:
        assert "stackTrace" in entry
        assert isinstance(entry["stackTrace"], object)
        assert "callFrames" in entry["stackTrace"]

        call_frames = entry["stackTrace"]["callFrames"]
        assert isinstance(call_frames, list)
        assert len(call_frames) == len(stacktrace)
        for index in range(0, len(call_frames)):
            assert call_frames[index] == stacktrace[index]

    source = entry["source"]
    if context is not None:
        assert "context" in source
        assert source["context"] == context


def assert_console_entry(
    entry,
    method=any_string,
    level=any_string,
    text=any_string,
    args=any_list,
    timestamp=any_int,
    realm=any_string,
    context=None,
    stacktrace=None
):
    assert_base_entry(
        entry=entry,
        level=level,
        text=text,
        timestamp=timestamp,
        realm=realm,
        context=context,
        stacktrace=stacktrace)

    recursive_compare({
        "type": "console",
        "method": method,
        "args": args
    }, entry)


def assert_javascript_entry(
    entry,
    level=any_string,
    text=any_string,
    timestamp=any_int,
    realm=any_string,
    context=None,
    stacktrace=None
):
    assert_base_entry(
        entry=entry,
        level=level,
        text=text,
        timestamp=timestamp,
        realm=realm,
        stacktrace=stacktrace,
        context=context)

    recursive_compare({
        "type": "javascript",
    }, entry)


async def create_console_api_message_from_string(bidi_session, context, type, value):
    await bidi_session.script.evaluate(
        expression=f"""console.{type}({value})""",
        await_promise=False,
        target=ContextTarget(context["context"]),
    )


async def create_javascript_error(bidi_session, context, error_message="foo"):
    str_remote_value = {"type": "string", "value": error_message}

    result = await bidi_session.script.call_function(
        function_declaration="""(error_message) => {
            const script = document.createElement("script");
            script.append(document.createTextNode(`(() => { throw new Error("${error_message}") })()`));
            document.body.append(script);

            const err = new Error(error_message);
            return err.toString();
        }""",
        arguments=[str_remote_value],
        await_promise=False,
        target=ContextTarget(context["context"]),
    )

    return result["value"]


def create_log(bidi_session, context, log_type, text="foo"):
    if log_type == "console_api_log":
        return create_console_api_message(bidi_session, context, text)
    if log_type == "javascript_error":
        return create_javascript_error(bidi_session, context, text)
