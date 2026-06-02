{% for section in sections %}
{% set underline = "-" %}
{% if section %}
{{section}}
{{ underline * section|length }}{% set underline = "~" %}

{% endif %}
{% if sections[section] %}
{% for category, val in definitions.items() if category in sections[section] %}

{{ definitions[category]['name'] }}
{{ underline * definitions[category]['name']|length }}

{% if definitions[category]['showcontent'] %}
{% for text, values in sections[section][category]|dictsort(by='value') %}
{% set issue_joiner = joiner(', ') %}
- {% for value in values|sort %}{{ issue_joiner() }}`{{ value }} <https://github.com/pytest-dev/pytest/issues/{{ value[1:] }}>`_{% endfor %}: {{ text }}


{% endfor %}
{% else %}
- {{ sections[section][category]['']|sort|join(', ') }}


{% endif %}
{% if sections[section][category]|length == 0 %}

No significant changes.


{% else %}
{% endif %}
{% endfor %}
{% else %}

No significant changes.


{% endif %}
{% endfor %}
