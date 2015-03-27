#!/usr/bin/env ruby
require 'nokogiri'

def base_dir
  File.dirname(__FILE__)
end

def output_directory
  File.join(base_dir, 'idl')
end

def specification
  file = File.open(File.join(base_dir, 'specification.html'))
  doc = Nokogiri::XML(file)
  file.close
  doc
end

def write_node_inner_text_to_file(filename, node)
  File.open(filename, 'w') { |file| file.write(node.inner_text.strip) }
  puts "Wrote: #{filename}"
end

def load_idl(id)
  file = File.join(output_directory, id)
  return false if !File.exist?(file)
  File.read(file)
end

# Parse the specification writing each block of idl to its own file
specification.css(".idl-code").each do |idl_block|
  id = idl_block["id"]
  write_node_inner_text_to_file(File.join(output_directory, id), idl_block) if id
end

# Update the idl in the pre blocks for each idl test
idl_test_files = [
  File.join(base_dir, 'the-audio-api', 'the-gainnode-interface', 'idl-test.html'),
  File.join(base_dir, 'the-audio-api', 'the-audiodestinationnode-interface', 'idl-test.html'),
  File.join(base_dir, 'the-audio-api', 'the-delaynode-interface', 'idl-test.html'),
  File.join(base_dir, 'the-audio-api', 'the-audiobuffer-interface', 'idl-test.html'),
]

idl_test_files.each do |fn|
  file = File.open(fn)
  doc = Nokogiri::HTML(file)
  file.close

  doc.css('pre').each do |node|
    node_id = node["id"]
    if idl = load_idl(node_id)
      node.content = idl
    end
  end

  File.open(fn, 'w') { |file| file.write(doc.to_html)}
end
