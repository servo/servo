from webdriver.bidi.modules.script import ContextTarget


async def get_bluetooth_availability(bidi_session, context):
    result = await bidi_session.script.evaluate(
        expression="navigator.bluetooth.getAvailability()",
        target=ContextTarget(context["context"]), await_promise=True, )
    return result['value']


async def set_simulate_adapter(bidi_session, context, test_page, state):
    # Navigate to a page, as bluetooth is not guaranteed to work on
    # `about:blank`.
    await bidi_session.browsing_context.navigate(context=context['context'],
                                                 url=test_page, wait="complete")

    await bidi_session.bluetooth.simulate_adapter(context=context["context"],
                                                  state=state)
