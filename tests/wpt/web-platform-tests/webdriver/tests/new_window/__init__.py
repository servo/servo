def opener(session):
    return session.execute_script("""
        return window.opener;
        """)


def window_name(session):
    return session.execute_script("""
        return window.name;
        """)
