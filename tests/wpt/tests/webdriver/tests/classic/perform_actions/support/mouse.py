def get_viewport_rect(session):
    return session.execute_script("""
        return {
          height: window.innerHeight || document.documentElement.clientHeight,
          width: window.innerWidth || document.documentElement.clientWidth,
        };
    """)


def get_inview_center(elem_rect, viewport_rect):
    x = {
        "left": max(0, min(elem_rect["x"], elem_rect["x"] + elem_rect["width"])),
        "right": min(viewport_rect["width"], max(elem_rect["x"],
                                                 elem_rect["x"] + elem_rect["width"])),
    }

    y = {
        "top": max(0, min(elem_rect["y"], elem_rect["y"] + elem_rect["height"])),
        "bottom": min(viewport_rect["height"], max(elem_rect["y"],
                                                   elem_rect["y"] + elem_rect["height"])),
    }

    return {
        "x": (x["left"] + x["right"]) / 2,
        "y": (y["top"] + y["bottom"]) / 2,
    }
