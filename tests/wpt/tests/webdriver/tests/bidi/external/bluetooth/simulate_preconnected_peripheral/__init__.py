async def set_simulate_preconnected_peripheral(bidi_session, context, test_page,
                                               address, name, manufacturer_data,
                                               known_service_uuids):
    # Navigate to a page, as bluetooth is not guaranteed to work on
    # `about:blank`.
    await bidi_session.browsing_context.navigate(context=context['context'],
                                                 url=test_page, wait="complete")
    await bidi_session.bluetooth.simulate_adapter(context=context["context"],
                                                  state="powered-on", type_="create")
    await bidi_session.bluetooth.simulate_preconnected_peripheral(
        context=context["context"],
        address=address, name=name,
        manufacturer_data=manufacturer_data,
        known_service_uuids=known_service_uuids)
