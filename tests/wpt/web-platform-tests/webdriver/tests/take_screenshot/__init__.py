def document_dimensions(session):
    return tuple(session.execute_script("return [window.innerWidth, window.innerHeight];"))
