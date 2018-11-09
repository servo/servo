{% set bin = "/usr/local/bin" %}
{% set keyfile = "/etc/generic-worker/key" %}

{{ bin }}/generic-worker:
  file.managed:
    - name:
    - source: https://github.com/taskcluster/generic-worker/releases/download/v11.0.1/generic-worker-darwin-amd64
    - source_hash: sha256=059331865670d3722a710f0b6f4dae97d347811cc347d1810c6dfc1b413c4b48
    - mode: 755
    - makedirs: True

{{ bin }}/livelog:
  file.managed:
    - source: https://github.com/taskcluster/livelog/releases/download/v1.1.0/livelog-darwin-amd64
    - source_hash: sha256=be5d4b998b208afd802ac6ce6c4d4bbf0fb3816bb039a300626abbc999dfe163
    - mode: 755
    - makedirs: True

/etc/generic-worker/config.json:
  file.serialize:
    - makedirs: True
    - mode: 600
    - show_changes: False
    - formatter: json
    - dataset:
        provisionerId: proj-servo
        workerType: macos
        workerId: servo-macos-1
        clientId: project/servo/worker/macos/1
        publicIP: {{ salt.network.ip_addrs()[0] }}
        signingKeyLocation: {{ keyfile }}

generic-worker new-openpgp-keypair --file {{ keyfile }}:
  cmd.run:
    - creates: {{ keyfile }}
    - prepend_path: {{ bin }}
