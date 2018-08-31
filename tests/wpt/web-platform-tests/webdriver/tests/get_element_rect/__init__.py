def retrieve_element_rect(session, element):
    return session.execute_script("""
        let rect = arguments[0].getBoundingClientRect();
        return {
            x: rect.left + window.pageXOffset,
            y: rect.top + window.pageYOffset,
            width: rect.width,
            height: rect.height,
        };
        """, args=(element,))
