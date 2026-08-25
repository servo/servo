from webdriver.bidi.modules.script import ContextTarget


async def get_bluetooth_availability(bidi_session, context):
    result = await bidi_session.script.evaluate(
        expression="navigator.bluetooth.getAvailability()",
        target=ContextTarget(context["context"]), await_promise=True, )
    return result['value']

