def map_files_to_multiline_text(files):
    return "\n".join(map(lambda f: str(f), files))
