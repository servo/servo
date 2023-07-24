
## Breakdown of AccName Name Computation files...

Portions of the AccName algorithm are referenced via unique IDs such as `comp_labelledby` and `comp_embedded_control`. This ReadMe lists those sections (and subsections) in order as they appear in [AccName Computation Steps](https://w3c.github.io/accname/#computation-steps).

In order to make the WPT test files digestible and understandable, the tests are broken up more or less in the structure of the algorithm, with the file struction listed below. Sub-section test (such as `comp_labelledby_recursion`) are tested as part of the main section `comp_labelledby` in [comp_labelledby.html](comp_labelledby.html).

Non-name portions of the AccName spec (such as Descripton Computation) should be tested in another directory.

If a new section of the AccName algorithm is added, please list it here when checking in new tests. Thanks.

### No-Op (no test files)
- comp_init
- comp_computation

### [comp_hidden_not_referenced](comp_hidden_not_referenced.html)

### [comp_labelledby](comp_labelledby.html)
  - comp_labelledby_reset
  - comp_labelledby_foreach
  - comp_labelledby_set_current
  - comp_labelledby_recursion
  - comp_labelledby_append
  - comp_labelledby_return

### [comp_embedded_control](comp_embedded_control.html)
 - comp_embedded_control_textbox
 - comp_embedded_control_combobox_or_listbox
 - comp_embedded_control_range
 - comp_embedded_control_range_valuetext
 - comp_embedded_control_range_valuenow
 - comp_embedded_control_range_host_language_value

### [comp_label](comp_label.html)

### [comp_host_language_label](comp_host_language_label.html)

### [comp_name_from_content](comp_name_from_content.html)
  - comp_name_from_content_reset
  - comp_name_from_content_pseudo_element
  - comp_name_from_content_pseudo_element_before
  - comp_name_from_content_pseudo_element_after
  - comp_name_from_content_for_each_child
  - comp_name_from_content_for_each_child_set_current
  - comp_name_from_content_for_each_child_recursion
  - comp_for_each_child_append
  - comp_name_from_content_return

### [comp_text_node](comp_text_node.html)

### comp_recursive_name_from_content (tested with [comp_name_from_content](comp_name_from_content.html))

### [comp_tooltip](comp_tooltip.html)

### No-Op (no test files)
  - comp_append
  - comp_complete



