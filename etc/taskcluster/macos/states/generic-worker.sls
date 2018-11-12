{% set bin = "/usr/local/bin" %}
{% set etc = "/etc/generic-worker" %}
{% set user = "worker" %}
{% set home = "/Users/" + user %}

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

{{ user }} group:
  group.present:
    - name: {{ user }}

{{ user }}:
  user.present:
    - home: {{ home }}
    - gid_from_name: True

# `user.present`’s `createhome` is apparently not supported on macOS
{{ home }}:
  file.directory:
    - user: {{ user }}

{{ etc }}/config.json:
  file.serialize:
    - makedirs: True
    - group: {{ user }}
    - mode: 640
    - show_changes: False
    - formatter: json
    - dataset:
        provisionerId: proj-servo
        workerType: macos
        workerGroup: servo-macos
        workerId: mac1
        tasksDir: {{ home }}/tasks
        publicIP: {{ salt.network.ip_addrs()[0] }}
        signingKeyLocation: {{ home }}/key
        clientId: {{ pillar["client_id"] }}
        accessToken: {{ pillar["access_token"] }}
        livelogExecutable: {{ bin }}/livelog
        livelogCertificate: {{ etc }}/livelog.crt
        livelogKey: {{ etc }}/livelog.key
        livelogSecret: {{ pillar["livelog_secret"] }}
    - watch_in:
      - service: net.generic.worker

{{ etc }}/livelog.crt:
  file.managed:
    - contents_pillar: livelog_cert
    - group: {{ user }}
    - mode: 640

{{ etc }}/livelog.key:
  file.managed:
    - contents_pillar: livelog_key
    - group: {{ user }}
    - mode: 640

{{ bin }}/generic-worker new-openpgp-keypair --file {{ home }}/key:
  cmd.run:
    - creates: {{ home }}/key
    - runas: {{ user }}

/Library/LaunchAgents/net.generic.worker.plist:
  file.managed:
    - mode: 644
    - template: jinja
    - source: salt://generic-worker.plist.jinja
    - context:
      bin: {{ bin }}
      etc: {{ etc }}
      home: {{ home }}
      user: {{ user }}

net.generic.worker:
  service.running:
    - enable: True
