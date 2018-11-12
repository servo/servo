{% set bin = "/usr/local/bin" %}
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

{{ user }}:
  user.present:
    - home: {{ home }}

# `user.present`’s `createhome` is apparently not supported on macOS
{{ home }}:
  file.directory:
    - user: {{ user }}

{{ home }}/config.json:
  file.serialize:
    - user: {{ user }}
    - mode: 600
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
        livelogCertificate: {{ home }}/livelog.crt
        livelogKey: {{ home }}/livelog.key
        livelogSecret: {{ pillar["livelog_secret"] }}
    - watch_in:
      - service: net.generic.worker

{{ home }}/livelog.crt:
  file.managed:
    - contents_pillar: livelog_cert
    - user: {{ user }}
    - mode: 600

{{ home }}/livelog.key:
  file.managed:
    - contents_pillar: livelog_key
    - user: {{ user }}
    - mode: 600

{{ bin }}/generic-worker new-openpgp-keypair --file {{ home }}/key:
  cmd.run:
    - creates: {{ home }}/key
    - runas: worker

{{ home }}/run:
  file.managed:
    - mode: 744
    - user: {{ user }}
    - template: jinja
    - contents: |-
        #!/bin/sh
        # generic-worker overwrites its config file to fill in defaults,
        # but we want to avoid touching config.json here
        # so that SaltStack knows to (only) restart the service when it (really) changes.
        cp -a config.json config-run.json
        exec {{ bin }}/generic-worker run --config config-run.json

/Library/LaunchAgents/net.generic.worker.plist:
  file.managed:
    - mode: 644
    - template: jinja
    - contents: >-
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
          <key>Label</key>
          <string>net.generic.worker</string>

          <key>ProgramArguments</key>
          <array>
            <string>{{ home }}/run</string>
          </array>

          <key>KeepAlive</key>
          <true/>

          <key>WorkingDirectory</key>
          <string>{{ home }}</string>

          <key>UserName</key>
          <string>{{ user }}</string>

          <key>StandardOutPath</key>
          <string>stdout.log</string>

          <key>StandardErrorPath</key>
          <string>stderr.log</string>
        </dict>
        </plist>

net.generic.worker:
  service.running:
    - enable: True
