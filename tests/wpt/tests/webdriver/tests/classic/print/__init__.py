def do_print(session, options={}):
    params = {}

    if options.get("background", None) is not None:
        params["background"] = options["background"]
    if options.get("margin", None) is not None:
        params["margin"] = options["margin"]
    if options.get("orientation") is not None:
        params["orientation"] = options["orientation"]
    if options.get("page") is not None:
        params["page"] = options["page"]
    if options.get("pageRanges") is not None:
        params["pageRanges"] = options["pageRanges"]
    if options.get("scale") is not None:
        params["scale"] = options["scale"]
    if options.get("shrinkToFit") is not None:
        params["shrinkToFit"] = options["shrinkToFit"]

    return session.transport.send(
        "POST", "session/{session_id}/print".format(**vars(session)), params
    )
