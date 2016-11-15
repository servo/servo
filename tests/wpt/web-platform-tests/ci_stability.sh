set -e

hosts_fixup() {
    echo "travis_fold:start:hosts_fixup"
    echo "Rewriting hosts file"
    echo "## /etc/hosts ##"
    cat /etc/hosts
    sudo sed -i 's/^::1\s*localhost/::1/' /etc/hosts
    sudo sh -c 'echo "
127.0.0.1 web-platform.test
127.0.0.1 www.web-platform.test
127.0.0.1 www1.web-platform.test
127.0.0.1 www2.web-platform.test
127.0.0.1 xn--n8j6ds53lwwkrqhv28a.web-platform.test
127.0.0.1 xn--lve-6lad.web-platform.test
" >> /etc/hosts'
    echo "== /etc/hosts =="
    cat /etc/hosts
    echo "----------------"
    echo "travis_fold:end:hosts_fixup"
}


test_stability() {
    python check_stability.py $PRODUCT
}

main() {
    hosts_fixup
    test_stability
}

main
