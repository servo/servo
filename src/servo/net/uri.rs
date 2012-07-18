export uri, build_uri;

type uri = {
    spec: ~str,
    scheme: ~str,
    host: option<~str>,
    port: option<uint>,
    path: ~str
};

fn build_uri(_spec: ~str) -> uri {
    fail
}
