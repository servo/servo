<!DOCTYPE html>
<html lang="en-US">
  <head>
    <meta charset="utf-8" />
    
  <link rel="shortcut icon" href="/static/icons/mimetypes/py.5ef6367a.png" />

    <title>test_exposed_extended_attribute.py - DXR</title>

    
  
      <link href="/static/css/dxr.75151d98.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/forms.583492b2.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/icons.6b0c33c8.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/code-style.a87614e9.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/selector-common.949000e3.css" rel="stylesheet" type="text/css" media="screen" />
      <link href="/static/css/filter.b696a09b.css" rel="stylesheet" type="text/css" media="screen" />
    
  <link href="/static/css/tree-selector.aeb25cea.css" rel="stylesheet" type="text/css" media="screen" />

  </head>
  <body>

    <div class="header">
        <div class="help-icon">
            <div class="help-msg">
                <p>DXR is a code search and navigation tool aimed at making sense of large projects. It supports full-text and regex searches as well as structural queries.</p>
                <ul>
                    <li><a href="https://github.com/mozilla/dxr">DXR on GitHub</a></li>
                    <li><a href="https://dxr.readthedocs.org/en/latest/">How to Get Involved</a></li>
                </ul>
            </div>
        </div>
        <form method="get" action="/mozilla-central/search" id="basic_search" class="search-box">
            <fieldset>
                <div id="search-box" class="flex-container" role="group">
                    <div class="elem_container find">
                        <label for="query" class="query_label visually-hidden">Find</label>
                        <input type="text" name="q" value="" maxlength="2048" id="query" class="query" accesskey="s" title="Search" placeholder="Search mozilla-central" autocomplete="off" />
                        <div class="zero-size-container">
                          <div class="bubble">
                          </div>
                        </div>
                        <section id="search-filter" class="search-filter">
                          <button type="button" class="sf-select-trigger" aria-label="Select Filter">
                              <!-- arrow icon using icon font -->
                              <span aria-hidden="true" data-filter-arrow="&#xe801;" class="sf-selector-arrow">
                                  Operators
                              </span>
                          </button>
                        </section>
                        <div class="sf-select-options sf-modal" aria-expanded="false">
                          <ul class="selector-options" tabindex="-1">
                            
                              <li>
                                <a href="javascript:void(0)" data-value="ext:">
                                  <span class="selector-option-label">
                                    ext
                                  </span>
                                  <span class="selector-option-description">
                                    Filename extension: <code>ext:cpp</code>. Always case-sensitive.
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="id:">
                                  <span class="selector-option-label">
                                    id
                                  </span>
                                  <span class="selector-option-description">
                                    Definition of an identifier: <code>id:someFunction</code> <code>id:SomeClass</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="path:">
                                  <span class="selector-option-label">
                                    path
                                  </span>
                                  <span class="selector-option-description">
                                    File or directory sub-path to search within. <code>*</code>, <code>?</code>, and <code>[...]</code> act as shell wildcards.
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="ref:">
                                  <span class="selector-option-label">
                                    ref
                                  </span>
                                  <span class="selector-option-description">
                                    Reference to an identifier: <code>ref:someVar</code> <code>ref:someType</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="regexp:">
                                  <span class="selector-option-label">
                                    regexp
                                  </span>
                                  <span class="selector-option-description">
                                    Regular expression. Examples: <code>regexp:(?i)\bs?printf</code> <code>regexp:"(three|3) mice"</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="bases:">
                                  <span class="selector-option-label">
                                    bases
                                  </span>
                                  <span class="selector-option-description">
                                    Superclasses of a class: <code>bases:SomeSubclass</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="callers:">
                                  <span class="selector-option-label">
                                    callers
                                  </span>
                                  <span class="selector-option-description">
                                    Calls to the given function: <code>callers:some_function</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="derived:">
                                  <span class="selector-option-label">
                                    derived
                                  </span>
                                  <span class="selector-option-description">
                                    Subclasses of a class: <code>derived:SomeSuperclass</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="extern-ref:">
                                  <span class="selector-option-label">
                                    extern-ref
                                  </span>
                                  <span class="selector-option-description">
                                    References to items in external crate
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="fn-impls:">
                                  <span class="selector-option-label">
                                    fn-impls
                                  </span>
                                  <span class="selector-option-description">
                                    Function implementations
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="function:">
                                  <span class="selector-option-label">
                                    function
                                  </span>
                                  <span class="selector-option-description">
                                    Function or method definition: <code>function:foo</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="function-decl:">
                                  <span class="selector-option-label">
                                    function-decl
                                  </span>
                                  <span class="selector-option-description">
                                    Function or method declaration
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="function-ref:">
                                  <span class="selector-option-label">
                                    function-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Function or method references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="impl:">
                                  <span class="selector-option-label">
                                    impl
                                  </span>
                                  <span class="selector-option-description">
                                    Implementations
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="macro:">
                                  <span class="selector-option-label">
                                    macro
                                  </span>
                                  <span class="selector-option-description">
                                    Macro definition
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="macro-ref:">
                                  <span class="selector-option-label">
                                    macro-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Macro uses
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="member:">
                                  <span class="selector-option-label">
                                    member
                                  </span>
                                  <span class="selector-option-description">
                                    Member variables, types, or methods of a class: <code>member:SomeClass</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="module:">
                                  <span class="selector-option-label">
                                    module
                                  </span>
                                  <span class="selector-option-description">
                                    Module definition: <code>module:module.name</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="module-alias-ref:">
                                  <span class="selector-option-label">
                                    module-alias-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Module alias references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="module-ref:">
                                  <span class="selector-option-label">
                                    module-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Module references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="module-use:">
                                  <span class="selector-option-label">
                                    module-use
                                  </span>
                                  <span class="selector-option-description">
                                    Module imports
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="namespace:">
                                  <span class="selector-option-label">
                                    namespace
                                  </span>
                                  <span class="selector-option-description">
                                    Namespace definition
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="namespace-alias:">
                                  <span class="selector-option-label">
                                    namespace-alias
                                  </span>
                                  <span class="selector-option-description">
                                    Namespace alias
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="namespace-alias-ref:">
                                  <span class="selector-option-label">
                                    namespace-alias-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Namespace alias references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="namespace-ref:">
                                  <span class="selector-option-label">
                                    namespace-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Namespace references
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="overridden:">
                                  <span class="selector-option-label">
                                    overridden
                                  </span>
                                  <span class="selector-option-description">
                                    Methods which are overridden by the given one. Useful mostly with fully qualified methods, like <code>+overridden:foo.bar.some_method</code>.
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="overrides:">
                                  <span class="selector-option-label">
                                    overrides
                                  </span>
                                  <span class="selector-option-description">
                                    Methods which override the given one: <code>overrides:some_method</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="type:">
                                  <span class="selector-option-label">
                                    type
                                  </span>
                                  <span class="selector-option-description">
                                    Class definition: <code>type:Stack</code>
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="type-decl:">
                                  <span class="selector-option-label">
                                    type-decl
                                  </span>
                                  <span class="selector-option-description">
                                    Type or class declaration
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="type-ref:">
                                  <span class="selector-option-label">
                                    type-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Type or class references, uses, or instantiations
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="var:">
                                  <span class="selector-option-label">
                                    var
                                  </span>
                                  <span class="selector-option-description">
                                    Variable definition
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="var-decl:">
                                  <span class="selector-option-label">
                                    var-decl
                                  </span>
                                  <span class="selector-option-description">
                                    Variable declaration
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="var-ref:">
                                  <span class="selector-option-label">
                                    var-ref
                                  </span>
                                  <span class="selector-option-description">
                                    Variable uses (lvalue, rvalue, dereference, etc.)
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="warning:">
                                  <span class="selector-option-label">
                                    warning
                                  </span>
                                  <span class="selector-option-description">
                                    Compiler warning messages
                                  </span>
                                </a>
                              </li>
                            
                              <li>
                                <a href="javascript:void(0)" data-value="warning-opt:">
                                  <span class="selector-option-label">
                                    warning-opt
                                  </span>
                                  <span class="selector-option-description">
                                    Warning messages brought on by a given compiler command-line option
                                  </span>
                                </a>
                              </li>
                            
                          </ul>
                        </div>
                    </div>

                    <div class="elem_container case">
                        <label for="case">
                            <input type="checkbox" name="case" id="case" class="checkbox_case" value="true" accesskey="c" /><span class="access-key">C</span>ase-sensitive
                        </label>
                    </div>
                </div>
            </fieldset>

            <input type="hidden" value="mozilla-central" id="ts-value" />
            <input type="hidden" name="redirect" value="true" id="redirect" />
            <input type="submit" value="Search" class="visually-hidden" />
        </form>
    </div>

    <div id="content" class="content">
      
  
  <div class="breadcrumbs"><a href="/mozilla-central/source">mozilla-central</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom">dom</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings">bindings</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser">parser</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser/tests">tests</a><span class="path-separator">/</span><a href="/mozilla-central/source/dom/bindings/parser/tests/test_exposed_extended_attribute.py">test_exposed_extended_attribute.py</a></div>

  
  
    <section id="tree-selector" class="tree-selector">
      <button type="button" class="ts-select-trigger" aria-label="Switch Tree">
        <!-- arrow icon using icon font -->
        <span aria-hidden="true" data-icon-arrow="&#xe801;" class="selector-arrow">
          <!-- tree icon using icon font -->
          <span aria-hidden="true" data-icon="&#xe800;"></span>
          <span class='current-tree'>Switch Tree</span>
        </span>
      </button>
      <div class="select-options ts-modal" aria-expanded="false">
        <form name="options-filter" class="options-filter" data-active="false">
          <label for="filter-txt" class="visually-hidden">Filter Trees</label>
          <input type="text" name="filter-txt" id="filter-txt" placeholder="Filter trees" />
          <input type="submit" value="Filter" class="visually-hidden" />
        </form>
        <ul class="selector-options" tabindex="-1">
          
            <li>
              <a href="/build-central/parallel/dom/bindings/parser/tests/test_exposed_extended_attribute.py" >
                <span class="selector-option-label">build-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/comm-central/parallel/dom/bindings/parser/tests/test_exposed_extended_attribute.py" >
                <span class="selector-option-label">comm-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/hgcustom_version-control-tools/parallel/dom/bindings/parser/tests/test_exposed_extended_attribute.py" >
                <span class="selector-option-label">hgcustom_version-control-tools</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/mozilla-central/parallel/dom/bindings/parser/tests/test_exposed_extended_attribute.py" class="selected" aria-checked="true">
                <span class="selector-option-label">mozilla-central</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/nss/parallel/dom/bindings/parser/tests/test_exposed_extended_attribute.py" >
                <span class="selector-option-label">nss</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/rust/parallel/dom/bindings/parser/tests/test_exposed_extended_attribute.py" >
                <span class="selector-option-label">rust</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
            <li>
              <a href="/rustfmt/parallel/dom/bindings/parser/tests/test_exposed_extended_attribute.py" >
                <span class="selector-option-label">rustfmt</span>
                <span class="selector-option-description"></span>
              </a>
            </li>
          
        </ul>
      </div>
    </section>
  



  

  
    <div class="panel">
      <button id="panel-toggle">
        <span class="navpanel-icon expanded" aria-hidden="false"></span>
        Navigation
      </button>
      <section id="panel-content" aria-expanded="true" aria-hidden="false">
        
          <h4>Mercurial (05c087337043)</h4>
          <ul>
            
              <li>
                <a href="/mozilla-central/rev/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_exposed_extended_attribute.py" title="Permalink" class="permalink icon">Permalink</a>
              </li>
          </ul>
        
          <h4>Untracked file</h4>
          <ul>
            
          </ul>
        
          <h4>VCS Links</h4>
          <ul>
            
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/filelog/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_exposed_extended_attribute.py" title="Log" class="log icon">Log</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/annotate/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_exposed_extended_attribute.py" title="Blame" class="blame icon">Blame</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/diff/105d8ea002aa7565e83e9f6f8331d2e604bbd61d/dom/bindings/parser/tests/test_exposed_extended_attribute.py" title="Diff" class="diff icon">Diff</a>
              </li>
              <li>
                <a href="https://hg.mozilla.org/mozilla-central/raw-file/05c087337043dd8e71cc27bdb5b9d55fd00aaa26/dom/bindings/parser/tests/test_exposed_extended_attribute.py" title="Raw" class="raw icon">Raw</a>
              </li>
          </ul>
        
      </section>
    </div>
  

  <div id="annotations">
    
      <div class="annotation-set" id="aset-1"></div>
      <div class="annotation-set" id="aset-2"></div>
      <div class="annotation-set" id="aset-3"></div>
      <div class="annotation-set" id="aset-4"></div>
      <div class="annotation-set" id="aset-5"></div>
      <div class="annotation-set" id="aset-6"></div>
      <div class="annotation-set" id="aset-7"></div>
      <div class="annotation-set" id="aset-8"></div>
      <div class="annotation-set" id="aset-9"></div>
      <div class="annotation-set" id="aset-10"></div>
      <div class="annotation-set" id="aset-11"></div>
      <div class="annotation-set" id="aset-12"></div>
      <div class="annotation-set" id="aset-13"></div>
      <div class="annotation-set" id="aset-14"></div>
      <div class="annotation-set" id="aset-15"></div>
      <div class="annotation-set" id="aset-16"></div>
      <div class="annotation-set" id="aset-17"></div>
      <div class="annotation-set" id="aset-18"></div>
      <div class="annotation-set" id="aset-19"></div>
      <div class="annotation-set" id="aset-20"></div>
      <div class="annotation-set" id="aset-21"></div>
      <div class="annotation-set" id="aset-22"></div>
      <div class="annotation-set" id="aset-23"></div>
      <div class="annotation-set" id="aset-24"></div>
      <div class="annotation-set" id="aset-25"></div>
      <div class="annotation-set" id="aset-26"></div>
      <div class="annotation-set" id="aset-27"></div>
      <div class="annotation-set" id="aset-28"></div>
      <div class="annotation-set" id="aset-29"></div>
      <div class="annotation-set" id="aset-30"></div>
      <div class="annotation-set" id="aset-31"></div>
      <div class="annotation-set" id="aset-32"></div>
      <div class="annotation-set" id="aset-33"></div>
      <div class="annotation-set" id="aset-34"></div>
      <div class="annotation-set" id="aset-35"></div>
      <div class="annotation-set" id="aset-36"></div>
      <div class="annotation-set" id="aset-37"></div>
      <div class="annotation-set" id="aset-38"></div>
      <div class="annotation-set" id="aset-39"></div>
      <div class="annotation-set" id="aset-40"></div>
      <div class="annotation-set" id="aset-41"></div>
      <div class="annotation-set" id="aset-42"></div>
      <div class="annotation-set" id="aset-43"></div>
      <div class="annotation-set" id="aset-44"></div>
      <div class="annotation-set" id="aset-45"></div>
      <div class="annotation-set" id="aset-46"></div>
      <div class="annotation-set" id="aset-47"></div>
      <div class="annotation-set" id="aset-48"></div>
      <div class="annotation-set" id="aset-49"></div>
      <div class="annotation-set" id="aset-50"></div>
      <div class="annotation-set" id="aset-51"></div>
      <div class="annotation-set" id="aset-52"></div>
      <div class="annotation-set" id="aset-53"></div>
      <div class="annotation-set" id="aset-54"></div>
      <div class="annotation-set" id="aset-55"></div>
      <div class="annotation-set" id="aset-56"></div>
      <div class="annotation-set" id="aset-57"></div>
      <div class="annotation-set" id="aset-58"></div>
      <div class="annotation-set" id="aset-59"></div>
      <div class="annotation-set" id="aset-60"></div>
      <div class="annotation-set" id="aset-61"></div>
      <div class="annotation-set" id="aset-62"></div>
      <div class="annotation-set" id="aset-63"></div>
      <div class="annotation-set" id="aset-64"></div>
      <div class="annotation-set" id="aset-65"></div>
      <div class="annotation-set" id="aset-66"></div>
      <div class="annotation-set" id="aset-67"></div>
      <div class="annotation-set" id="aset-68"></div>
      <div class="annotation-set" id="aset-69"></div>
      <div class="annotation-set" id="aset-70"></div>
      <div class="annotation-set" id="aset-71"></div>
      <div class="annotation-set" id="aset-72"></div>
      <div class="annotation-set" id="aset-73"></div>
      <div class="annotation-set" id="aset-74"></div>
      <div class="annotation-set" id="aset-75"></div>
      <div class="annotation-set" id="aset-76"></div>
      <div class="annotation-set" id="aset-77"></div>
      <div class="annotation-set" id="aset-78"></div>
      <div class="annotation-set" id="aset-79"></div>
      <div class="annotation-set" id="aset-80"></div>
      <div class="annotation-set" id="aset-81"></div>
      <div class="annotation-set" id="aset-82"></div>
      <div class="annotation-set" id="aset-83"></div>
      <div class="annotation-set" id="aset-84"></div>
      <div class="annotation-set" id="aset-85"></div>
      <div class="annotation-set" id="aset-86"></div>
      <div class="annotation-set" id="aset-87"></div>
      <div class="annotation-set" id="aset-88"></div>
      <div class="annotation-set" id="aset-89"></div>
      <div class="annotation-set" id="aset-90"></div>
      <div class="annotation-set" id="aset-91"></div>
      <div class="annotation-set" id="aset-92"></div>
      <div class="annotation-set" id="aset-93"></div>
      <div class="annotation-set" id="aset-94"></div>
      <div class="annotation-set" id="aset-95"></div>
      <div class="annotation-set" id="aset-96"></div>
      <div class="annotation-set" id="aset-97"></div>
      <div class="annotation-set" id="aset-98"></div>
      <div class="annotation-set" id="aset-99"></div>
      <div class="annotation-set" id="aset-100"></div>
      <div class="annotation-set" id="aset-101"></div>
      <div class="annotation-set" id="aset-102"></div>
      <div class="annotation-set" id="aset-103"></div>
      <div class="annotation-set" id="aset-104"></div>
      <div class="annotation-set" id="aset-105"></div>
      <div class="annotation-set" id="aset-106"></div>
      <div class="annotation-set" id="aset-107"></div>
      <div class="annotation-set" id="aset-108"></div>
      <div class="annotation-set" id="aset-109"></div>
      <div class="annotation-set" id="aset-110"></div>
      <div class="annotation-set" id="aset-111"></div>
      <div class="annotation-set" id="aset-112"></div>
      <div class="annotation-set" id="aset-113"></div>
      <div class="annotation-set" id="aset-114"></div>
      <div class="annotation-set" id="aset-115"></div>
      <div class="annotation-set" id="aset-116"></div>
      <div class="annotation-set" id="aset-117"></div>
      <div class="annotation-set" id="aset-118"></div>
      <div class="annotation-set" id="aset-119"></div>
      <div class="annotation-set" id="aset-120"></div>
      <div class="annotation-set" id="aset-121"></div>
      <div class="annotation-set" id="aset-122"></div>
      <div class="annotation-set" id="aset-123"></div>
      <div class="annotation-set" id="aset-124"></div>
      <div class="annotation-set" id="aset-125"></div>
      <div class="annotation-set" id="aset-126"></div>
      <div class="annotation-set" id="aset-127"></div>
      <div class="annotation-set" id="aset-128"></div>
      <div class="annotation-set" id="aset-129"></div>
      <div class="annotation-set" id="aset-130"></div>
      <div class="annotation-set" id="aset-131"></div>
      <div class="annotation-set" id="aset-132"></div>
      <div class="annotation-set" id="aset-133"></div>
      <div class="annotation-set" id="aset-134"></div>
      <div class="annotation-set" id="aset-135"></div>
      <div class="annotation-set" id="aset-136"></div>
      <div class="annotation-set" id="aset-137"></div>
      <div class="annotation-set" id="aset-138"></div>
      <div class="annotation-set" id="aset-139"></div>
      <div class="annotation-set" id="aset-140"></div>
      <div class="annotation-set" id="aset-141"></div>
      <div class="annotation-set" id="aset-142"></div>
      <div class="annotation-set" id="aset-143"></div>
      <div class="annotation-set" id="aset-144"></div>
      <div class="annotation-set" id="aset-145"></div>
      <div class="annotation-set" id="aset-146"></div>
      <div class="annotation-set" id="aset-147"></div>
      <div class="annotation-set" id="aset-148"></div>
      <div class="annotation-set" id="aset-149"></div>
      <div class="annotation-set" id="aset-150"></div>
      <div class="annotation-set" id="aset-151"></div>
      <div class="annotation-set" id="aset-152"></div>
      <div class="annotation-set" id="aset-153"></div>
      <div class="annotation-set" id="aset-154"></div>
      <div class="annotation-set" id="aset-155"></div>
      <div class="annotation-set" id="aset-156"></div>
      <div class="annotation-set" id="aset-157"></div>
      <div class="annotation-set" id="aset-158"></div>
      <div class="annotation-set" id="aset-159"></div>
      <div class="annotation-set" id="aset-160"></div>
      <div class="annotation-set" id="aset-161"></div>
      <div class="annotation-set" id="aset-162"></div>
      <div class="annotation-set" id="aset-163"></div>
      <div class="annotation-set" id="aset-164"></div>
      <div class="annotation-set" id="aset-165"></div>
      <div class="annotation-set" id="aset-166"></div>
      <div class="annotation-set" id="aset-167"></div>
      <div class="annotation-set" id="aset-168"></div>
      <div class="annotation-set" id="aset-169"></div>
      <div class="annotation-set" id="aset-170"></div>
      <div class="annotation-set" id="aset-171"></div>
      <div class="annotation-set" id="aset-172"></div>
      <div class="annotation-set" id="aset-173"></div>
      <div class="annotation-set" id="aset-174"></div>
      <div class="annotation-set" id="aset-175"></div>
      <div class="annotation-set" id="aset-176"></div>
      <div class="annotation-set" id="aset-177"></div>
      <div class="annotation-set" id="aset-178"></div>
      <div class="annotation-set" id="aset-179"></div>
      <div class="annotation-set" id="aset-180"></div>
      <div class="annotation-set" id="aset-181"></div>
      <div class="annotation-set" id="aset-182"></div>
      <div class="annotation-set" id="aset-183"></div>
      <div class="annotation-set" id="aset-184"></div>
      <div class="annotation-set" id="aset-185"></div>
      <div class="annotation-set" id="aset-186"></div>
      <div class="annotation-set" id="aset-187"></div>
      <div class="annotation-set" id="aset-188"></div>
      <div class="annotation-set" id="aset-189"></div>
      <div class="annotation-set" id="aset-190"></div>
      <div class="annotation-set" id="aset-191"></div>
      <div class="annotation-set" id="aset-192"></div>
      <div class="annotation-set" id="aset-193"></div>
      <div class="annotation-set" id="aset-194"></div>
      <div class="annotation-set" id="aset-195"></div>
      <div class="annotation-set" id="aset-196"></div>
      <div class="annotation-set" id="aset-197"></div>
      <div class="annotation-set" id="aset-198"></div>
      <div class="annotation-set" id="aset-199"></div>
      <div class="annotation-set" id="aset-200"></div>
      <div class="annotation-set" id="aset-201"></div>
      <div class="annotation-set" id="aset-202"></div>
      <div class="annotation-set" id="aset-203"></div>
      <div class="annotation-set" id="aset-204"></div>
      <div class="annotation-set" id="aset-205"></div>
      <div class="annotation-set" id="aset-206"></div>
      <div class="annotation-set" id="aset-207"></div>
      <div class="annotation-set" id="aset-208"></div>
      <div class="annotation-set" id="aset-209"></div>
      <div class="annotation-set" id="aset-210"></div>
      <div class="annotation-set" id="aset-211"></div>
      <div class="annotation-set" id="aset-212"></div>
      <div class="annotation-set" id="aset-213"></div>
      <div class="annotation-set" id="aset-214"></div>
      <div class="annotation-set" id="aset-215"></div>
      <div class="annotation-set" id="aset-216"></div>
      <div class="annotation-set" id="aset-217"></div>
      <div class="annotation-set" id="aset-218"></div>
      <div class="annotation-set" id="aset-219"></div>
      <div class="annotation-set" id="aset-220"></div>
      <div class="annotation-set" id="aset-221"></div>
      <div class="annotation-set" id="aset-222"></div></div>

  <table id="file" class="file">
    <thead class="visually-hidden">
        <th scope="col">Line</th>
        <th scope="col">Code</th>
    </thead>
    <tbody>
      <tr>
        <td id="line-numbers">
          
            <span id="1" class="line-number" unselectable="on" rel="#1">1</span>
          
            <span id="2" class="line-number" unselectable="on" rel="#2">2</span>
          
            <span id="3" class="line-number" unselectable="on" rel="#3">3</span>
          
            <span id="4" class="line-number" unselectable="on" rel="#4">4</span>
          
            <span id="5" class="line-number" unselectable="on" rel="#5">5</span>
          
            <span id="6" class="line-number" unselectable="on" rel="#6">6</span>
          
            <span id="7" class="line-number" unselectable="on" rel="#7">7</span>
          
            <span id="8" class="line-number" unselectable="on" rel="#8">8</span>
          
            <span id="9" class="line-number" unselectable="on" rel="#9">9</span>
          
            <span id="10" class="line-number" unselectable="on" rel="#10">10</span>
          
            <span id="11" class="line-number" unselectable="on" rel="#11">11</span>
          
            <span id="12" class="line-number" unselectable="on" rel="#12">12</span>
          
            <span id="13" class="line-number" unselectable="on" rel="#13">13</span>
          
            <span id="14" class="line-number" unselectable="on" rel="#14">14</span>
          
            <span id="15" class="line-number" unselectable="on" rel="#15">15</span>
          
            <span id="16" class="line-number" unselectable="on" rel="#16">16</span>
          
            <span id="17" class="line-number" unselectable="on" rel="#17">17</span>
          
            <span id="18" class="line-number" unselectable="on" rel="#18">18</span>
          
            <span id="19" class="line-number" unselectable="on" rel="#19">19</span>
          
            <span id="20" class="line-number" unselectable="on" rel="#20">20</span>
          
            <span id="21" class="line-number" unselectable="on" rel="#21">21</span>
          
            <span id="22" class="line-number" unselectable="on" rel="#22">22</span>
          
            <span id="23" class="line-number" unselectable="on" rel="#23">23</span>
          
            <span id="24" class="line-number" unselectable="on" rel="#24">24</span>
          
            <span id="25" class="line-number" unselectable="on" rel="#25">25</span>
          
            <span id="26" class="line-number" unselectable="on" rel="#26">26</span>
          
            <span id="27" class="line-number" unselectable="on" rel="#27">27</span>
          
            <span id="28" class="line-number" unselectable="on" rel="#28">28</span>
          
            <span id="29" class="line-number" unselectable="on" rel="#29">29</span>
          
            <span id="30" class="line-number" unselectable="on" rel="#30">30</span>
          
            <span id="31" class="line-number" unselectable="on" rel="#31">31</span>
          
            <span id="32" class="line-number" unselectable="on" rel="#32">32</span>
          
            <span id="33" class="line-number" unselectable="on" rel="#33">33</span>
          
            <span id="34" class="line-number" unselectable="on" rel="#34">34</span>
          
            <span id="35" class="line-number" unselectable="on" rel="#35">35</span>
          
            <span id="36" class="line-number" unselectable="on" rel="#36">36</span>
          
            <span id="37" class="line-number" unselectable="on" rel="#37">37</span>
          
            <span id="38" class="line-number" unselectable="on" rel="#38">38</span>
          
            <span id="39" class="line-number" unselectable="on" rel="#39">39</span>
          
            <span id="40" class="line-number" unselectable="on" rel="#40">40</span>
          
            <span id="41" class="line-number" unselectable="on" rel="#41">41</span>
          
            <span id="42" class="line-number" unselectable="on" rel="#42">42</span>
          
            <span id="43" class="line-number" unselectable="on" rel="#43">43</span>
          
            <span id="44" class="line-number" unselectable="on" rel="#44">44</span>
          
            <span id="45" class="line-number" unselectable="on" rel="#45">45</span>
          
            <span id="46" class="line-number" unselectable="on" rel="#46">46</span>
          
            <span id="47" class="line-number" unselectable="on" rel="#47">47</span>
          
            <span id="48" class="line-number" unselectable="on" rel="#48">48</span>
          
            <span id="49" class="line-number" unselectable="on" rel="#49">49</span>
          
            <span id="50" class="line-number" unselectable="on" rel="#50">50</span>
          
            <span id="51" class="line-number" unselectable="on" rel="#51">51</span>
          
            <span id="52" class="line-number" unselectable="on" rel="#52">52</span>
          
            <span id="53" class="line-number" unselectable="on" rel="#53">53</span>
          
            <span id="54" class="line-number" unselectable="on" rel="#54">54</span>
          
            <span id="55" class="line-number" unselectable="on" rel="#55">55</span>
          
            <span id="56" class="line-number" unselectable="on" rel="#56">56</span>
          
            <span id="57" class="line-number" unselectable="on" rel="#57">57</span>
          
            <span id="58" class="line-number" unselectable="on" rel="#58">58</span>
          
            <span id="59" class="line-number" unselectable="on" rel="#59">59</span>
          
            <span id="60" class="line-number" unselectable="on" rel="#60">60</span>
          
            <span id="61" class="line-number" unselectable="on" rel="#61">61</span>
          
            <span id="62" class="line-number" unselectable="on" rel="#62">62</span>
          
            <span id="63" class="line-number" unselectable="on" rel="#63">63</span>
          
            <span id="64" class="line-number" unselectable="on" rel="#64">64</span>
          
            <span id="65" class="line-number" unselectable="on" rel="#65">65</span>
          
            <span id="66" class="line-number" unselectable="on" rel="#66">66</span>
          
            <span id="67" class="line-number" unselectable="on" rel="#67">67</span>
          
            <span id="68" class="line-number" unselectable="on" rel="#68">68</span>
          
            <span id="69" class="line-number" unselectable="on" rel="#69">69</span>
          
            <span id="70" class="line-number" unselectable="on" rel="#70">70</span>
          
            <span id="71" class="line-number" unselectable="on" rel="#71">71</span>
          
            <span id="72" class="line-number" unselectable="on" rel="#72">72</span>
          
            <span id="73" class="line-number" unselectable="on" rel="#73">73</span>
          
            <span id="74" class="line-number" unselectable="on" rel="#74">74</span>
          
            <span id="75" class="line-number" unselectable="on" rel="#75">75</span>
          
            <span id="76" class="line-number" unselectable="on" rel="#76">76</span>
          
            <span id="77" class="line-number" unselectable="on" rel="#77">77</span>
          
            <span id="78" class="line-number" unselectable="on" rel="#78">78</span>
          
            <span id="79" class="line-number" unselectable="on" rel="#79">79</span>
          
            <span id="80" class="line-number" unselectable="on" rel="#80">80</span>
          
            <span id="81" class="line-number" unselectable="on" rel="#81">81</span>
          
            <span id="82" class="line-number" unselectable="on" rel="#82">82</span>
          
            <span id="83" class="line-number" unselectable="on" rel="#83">83</span>
          
            <span id="84" class="line-number" unselectable="on" rel="#84">84</span>
          
            <span id="85" class="line-number" unselectable="on" rel="#85">85</span>
          
            <span id="86" class="line-number" unselectable="on" rel="#86">86</span>
          
            <span id="87" class="line-number" unselectable="on" rel="#87">87</span>
          
            <span id="88" class="line-number" unselectable="on" rel="#88">88</span>
          
            <span id="89" class="line-number" unselectable="on" rel="#89">89</span>
          
            <span id="90" class="line-number" unselectable="on" rel="#90">90</span>
          
            <span id="91" class="line-number" unselectable="on" rel="#91">91</span>
          
            <span id="92" class="line-number" unselectable="on" rel="#92">92</span>
          
            <span id="93" class="line-number" unselectable="on" rel="#93">93</span>
          
            <span id="94" class="line-number" unselectable="on" rel="#94">94</span>
          
            <span id="95" class="line-number" unselectable="on" rel="#95">95</span>
          
            <span id="96" class="line-number" unselectable="on" rel="#96">96</span>
          
            <span id="97" class="line-number" unselectable="on" rel="#97">97</span>
          
            <span id="98" class="line-number" unselectable="on" rel="#98">98</span>
          
            <span id="99" class="line-number" unselectable="on" rel="#99">99</span>
          
            <span id="100" class="line-number" unselectable="on" rel="#100">100</span>
          
            <span id="101" class="line-number" unselectable="on" rel="#101">101</span>
          
            <span id="102" class="line-number" unselectable="on" rel="#102">102</span>
          
            <span id="103" class="line-number" unselectable="on" rel="#103">103</span>
          
            <span id="104" class="line-number" unselectable="on" rel="#104">104</span>
          
            <span id="105" class="line-number" unselectable="on" rel="#105">105</span>
          
            <span id="106" class="line-number" unselectable="on" rel="#106">106</span>
          
            <span id="107" class="line-number" unselectable="on" rel="#107">107</span>
          
            <span id="108" class="line-number" unselectable="on" rel="#108">108</span>
          
            <span id="109" class="line-number" unselectable="on" rel="#109">109</span>
          
            <span id="110" class="line-number" unselectable="on" rel="#110">110</span>
          
            <span id="111" class="line-number" unselectable="on" rel="#111">111</span>
          
            <span id="112" class="line-number" unselectable="on" rel="#112">112</span>
          
            <span id="113" class="line-number" unselectable="on" rel="#113">113</span>
          
            <span id="114" class="line-number" unselectable="on" rel="#114">114</span>
          
            <span id="115" class="line-number" unselectable="on" rel="#115">115</span>
          
            <span id="116" class="line-number" unselectable="on" rel="#116">116</span>
          
            <span id="117" class="line-number" unselectable="on" rel="#117">117</span>
          
            <span id="118" class="line-number" unselectable="on" rel="#118">118</span>
          
            <span id="119" class="line-number" unselectable="on" rel="#119">119</span>
          
            <span id="120" class="line-number" unselectable="on" rel="#120">120</span>
          
            <span id="121" class="line-number" unselectable="on" rel="#121">121</span>
          
            <span id="122" class="line-number" unselectable="on" rel="#122">122</span>
          
            <span id="123" class="line-number" unselectable="on" rel="#123">123</span>
          
            <span id="124" class="line-number" unselectable="on" rel="#124">124</span>
          
            <span id="125" class="line-number" unselectable="on" rel="#125">125</span>
          
            <span id="126" class="line-number" unselectable="on" rel="#126">126</span>
          
            <span id="127" class="line-number" unselectable="on" rel="#127">127</span>
          
            <span id="128" class="line-number" unselectable="on" rel="#128">128</span>
          
            <span id="129" class="line-number" unselectable="on" rel="#129">129</span>
          
            <span id="130" class="line-number" unselectable="on" rel="#130">130</span>
          
            <span id="131" class="line-number" unselectable="on" rel="#131">131</span>
          
            <span id="132" class="line-number" unselectable="on" rel="#132">132</span>
          
            <span id="133" class="line-number" unselectable="on" rel="#133">133</span>
          
            <span id="134" class="line-number" unselectable="on" rel="#134">134</span>
          
            <span id="135" class="line-number" unselectable="on" rel="#135">135</span>
          
            <span id="136" class="line-number" unselectable="on" rel="#136">136</span>
          
            <span id="137" class="line-number" unselectable="on" rel="#137">137</span>
          
            <span id="138" class="line-number" unselectable="on" rel="#138">138</span>
          
            <span id="139" class="line-number" unselectable="on" rel="#139">139</span>
          
            <span id="140" class="line-number" unselectable="on" rel="#140">140</span>
          
            <span id="141" class="line-number" unselectable="on" rel="#141">141</span>
          
            <span id="142" class="line-number" unselectable="on" rel="#142">142</span>
          
            <span id="143" class="line-number" unselectable="on" rel="#143">143</span>
          
            <span id="144" class="line-number" unselectable="on" rel="#144">144</span>
          
            <span id="145" class="line-number" unselectable="on" rel="#145">145</span>
          
            <span id="146" class="line-number" unselectable="on" rel="#146">146</span>
          
            <span id="147" class="line-number" unselectable="on" rel="#147">147</span>
          
            <span id="148" class="line-number" unselectable="on" rel="#148">148</span>
          
            <span id="149" class="line-number" unselectable="on" rel="#149">149</span>
          
            <span id="150" class="line-number" unselectable="on" rel="#150">150</span>
          
            <span id="151" class="line-number" unselectable="on" rel="#151">151</span>
          
            <span id="152" class="line-number" unselectable="on" rel="#152">152</span>
          
            <span id="153" class="line-number" unselectable="on" rel="#153">153</span>
          
            <span id="154" class="line-number" unselectable="on" rel="#154">154</span>
          
            <span id="155" class="line-number" unselectable="on" rel="#155">155</span>
          
            <span id="156" class="line-number" unselectable="on" rel="#156">156</span>
          
            <span id="157" class="line-number" unselectable="on" rel="#157">157</span>
          
            <span id="158" class="line-number" unselectable="on" rel="#158">158</span>
          
            <span id="159" class="line-number" unselectable="on" rel="#159">159</span>
          
            <span id="160" class="line-number" unselectable="on" rel="#160">160</span>
          
            <span id="161" class="line-number" unselectable="on" rel="#161">161</span>
          
            <span id="162" class="line-number" unselectable="on" rel="#162">162</span>
          
            <span id="163" class="line-number" unselectable="on" rel="#163">163</span>
          
            <span id="164" class="line-number" unselectable="on" rel="#164">164</span>
          
            <span id="165" class="line-number" unselectable="on" rel="#165">165</span>
          
            <span id="166" class="line-number" unselectable="on" rel="#166">166</span>
          
            <span id="167" class="line-number" unselectable="on" rel="#167">167</span>
          
            <span id="168" class="line-number" unselectable="on" rel="#168">168</span>
          
            <span id="169" class="line-number" unselectable="on" rel="#169">169</span>
          
            <span id="170" class="line-number" unselectable="on" rel="#170">170</span>
          
            <span id="171" class="line-number" unselectable="on" rel="#171">171</span>
          
            <span id="172" class="line-number" unselectable="on" rel="#172">172</span>
          
            <span id="173" class="line-number" unselectable="on" rel="#173">173</span>
          
            <span id="174" class="line-number" unselectable="on" rel="#174">174</span>
          
            <span id="175" class="line-number" unselectable="on" rel="#175">175</span>
          
            <span id="176" class="line-number" unselectable="on" rel="#176">176</span>
          
            <span id="177" class="line-number" unselectable="on" rel="#177">177</span>
          
            <span id="178" class="line-number" unselectable="on" rel="#178">178</span>
          
            <span id="179" class="line-number" unselectable="on" rel="#179">179</span>
          
            <span id="180" class="line-number" unselectable="on" rel="#180">180</span>
          
            <span id="181" class="line-number" unselectable="on" rel="#181">181</span>
          
            <span id="182" class="line-number" unselectable="on" rel="#182">182</span>
          
            <span id="183" class="line-number" unselectable="on" rel="#183">183</span>
          
            <span id="184" class="line-number" unselectable="on" rel="#184">184</span>
          
            <span id="185" class="line-number" unselectable="on" rel="#185">185</span>
          
            <span id="186" class="line-number" unselectable="on" rel="#186">186</span>
          
            <span id="187" class="line-number" unselectable="on" rel="#187">187</span>
          
            <span id="188" class="line-number" unselectable="on" rel="#188">188</span>
          
            <span id="189" class="line-number" unselectable="on" rel="#189">189</span>
          
            <span id="190" class="line-number" unselectable="on" rel="#190">190</span>
          
            <span id="191" class="line-number" unselectable="on" rel="#191">191</span>
          
            <span id="192" class="line-number" unselectable="on" rel="#192">192</span>
          
            <span id="193" class="line-number" unselectable="on" rel="#193">193</span>
          
            <span id="194" class="line-number" unselectable="on" rel="#194">194</span>
          
            <span id="195" class="line-number" unselectable="on" rel="#195">195</span>
          
            <span id="196" class="line-number" unselectable="on" rel="#196">196</span>
          
            <span id="197" class="line-number" unselectable="on" rel="#197">197</span>
          
            <span id="198" class="line-number" unselectable="on" rel="#198">198</span>
          
            <span id="199" class="line-number" unselectable="on" rel="#199">199</span>
          
            <span id="200" class="line-number" unselectable="on" rel="#200">200</span>
          
            <span id="201" class="line-number" unselectable="on" rel="#201">201</span>
          
            <span id="202" class="line-number" unselectable="on" rel="#202">202</span>
          
            <span id="203" class="line-number" unselectable="on" rel="#203">203</span>
          
            <span id="204" class="line-number" unselectable="on" rel="#204">204</span>
          
            <span id="205" class="line-number" unselectable="on" rel="#205">205</span>
          
            <span id="206" class="line-number" unselectable="on" rel="#206">206</span>
          
            <span id="207" class="line-number" unselectable="on" rel="#207">207</span>
          
            <span id="208" class="line-number" unselectable="on" rel="#208">208</span>
          
            <span id="209" class="line-number" unselectable="on" rel="#209">209</span>
          
            <span id="210" class="line-number" unselectable="on" rel="#210">210</span>
          
            <span id="211" class="line-number" unselectable="on" rel="#211">211</span>
          
            <span id="212" class="line-number" unselectable="on" rel="#212">212</span>
          
            <span id="213" class="line-number" unselectable="on" rel="#213">213</span>
          
            <span id="214" class="line-number" unselectable="on" rel="#214">214</span>
          
            <span id="215" class="line-number" unselectable="on" rel="#215">215</span>
          
            <span id="216" class="line-number" unselectable="on" rel="#216">216</span>
          
            <span id="217" class="line-number" unselectable="on" rel="#217">217</span>
          
            <span id="218" class="line-number" unselectable="on" rel="#218">218</span>
          
            <span id="219" class="line-number" unselectable="on" rel="#219">219</span>
          
            <span id="220" class="line-number" unselectable="on" rel="#220">220</span>
          
            <span id="221" class="line-number" unselectable="on" rel="#221">221</span>
          
            <span id="222" class="line-number" unselectable="on" rel="#222">222</span>
          
        </td>
        <td class="code">
          
<pre>
<code id="line-1" aria-labelledby="1"><span class="k">import</span> WebIDL
</code><code id="line-2" aria-labelledby="2">
</code><code id="line-3" aria-labelledby="3"><span class="k">def</span> WebIDLTest(parser, harness):
</code><code id="line-4" aria-labelledby="4">    parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-5" aria-labelledby="5"><span class="str">      [PrimaryGlobal] interface Foo {};</span><span class="str">
</span></code><code id="line-6" aria-labelledby="6"><span class="str">      [Global=(Bar1,Bar2)] interface Bar {};</span><span class="str">
</span></code><code id="line-7" aria-labelledby="7"><span class="str">      [Global=Baz2] interface Baz {};</span><span class="str">
</span></code><code id="line-8" aria-labelledby="8"><span class="str">
</span></code><code id="line-9" aria-labelledby="9"><span class="str">      [Exposed=(Foo,Bar1)]</span><span class="str">
</span></code><code id="line-10" aria-labelledby="10"><span class="str">      interface Iface {</span><span class="str">
</span></code><code id="line-11" aria-labelledby="11"><span class="str">        void method1();</span><span class="str">
</span></code><code id="line-12" aria-labelledby="12"><span class="str">
</span></code><code id="line-13" aria-labelledby="13"><span class="str">        [Exposed=Bar1]</span><span class="str">
</span></code><code id="line-14" aria-labelledby="14"><span class="str">        readonly attribute any attr;</span><span class="str">
</span></code><code id="line-15" aria-labelledby="15"><span class="str">      };</span><span class="str">
</span></code><code id="line-16" aria-labelledby="16"><span class="str">
</span></code><code id="line-17" aria-labelledby="17"><span class="str">      [Exposed=Foo]</span><span class="str">
</span></code><code id="line-18" aria-labelledby="18"><span class="str">      partial interface Iface {</span><span class="str">
</span></code><code id="line-19" aria-labelledby="19"><span class="str">        void method2();</span><span class="str">
</span></code><code id="line-20" aria-labelledby="20"><span class="str">      };</span><span class="str">
</span></code><code id="line-21" aria-labelledby="21"><span class="str">    </span><span class="str">"""</span>)
</code><code id="line-22" aria-labelledby="22">
</code><code id="line-23" aria-labelledby="23">    results = parser.finish()
</code><code id="line-24" aria-labelledby="24">
</code><code id="line-25" aria-labelledby="25">    harness.check(len(results), 5, <span class="str">"</span><span class="str">Should know about five things</span><span class="str">"</span>);
</code><code id="line-26" aria-labelledby="26">    iface = results[3]
</code><code id="line-27" aria-labelledby="27">    harness.ok(isinstance(iface, WebIDL.IDLInterface),
</code><code id="line-28" aria-labelledby="28">               <span class="str">"</span><span class="str">Should have an interface here</span><span class="str">"</span>);
</code><code id="line-29" aria-labelledby="29">    members = iface.members
</code><code id="line-30" aria-labelledby="30">    harness.check(len(members), 3, <span class="str">"</span><span class="str">Should have three members</span><span class="str">"</span>)
</code><code id="line-31" aria-labelledby="31">
</code><code id="line-32" aria-labelledby="32">    harness.ok(members[0].exposureSet == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>, <span class="str">"</span><span class="str">Bar</span><span class="str">"</span>]),
</code><code id="line-33" aria-labelledby="33">               <span class="str">"</span><span class="str">method1 should have the right exposure set</span><span class="str">"</span>)
</code><code id="line-34" aria-labelledby="34">    harness.ok(members[0]._exposureGlobalNames == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>, <span class="str">"</span><span class="str">Bar1</span><span class="str">"</span>]),
</code><code id="line-35" aria-labelledby="35">               <span class="str">"</span><span class="str">method1 should have the right exposure global names</span><span class="str">"</span>)
</code><code id="line-36" aria-labelledby="36">
</code><code id="line-37" aria-labelledby="37">    harness.ok(members[1].exposureSet == set([<span class="str">"</span><span class="str">Bar</span><span class="str">"</span>]),
</code><code id="line-38" aria-labelledby="38">               <span class="str">"</span><span class="str">attr should have the right exposure set</span><span class="str">"</span>)
</code><code id="line-39" aria-labelledby="39">    harness.ok(members[1]._exposureGlobalNames == set([<span class="str">"</span><span class="str">Bar1</span><span class="str">"</span>]),
</code><code id="line-40" aria-labelledby="40">               <span class="str">"</span><span class="str">attr should have the right exposure global names</span><span class="str">"</span>)
</code><code id="line-41" aria-labelledby="41">
</code><code id="line-42" aria-labelledby="42">    harness.ok(members[2].exposureSet == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>]),
</code><code id="line-43" aria-labelledby="43">               <span class="str">"</span><span class="str">method2 should have the right exposure set</span><span class="str">"</span>)
</code><code id="line-44" aria-labelledby="44">    harness.ok(members[2]._exposureGlobalNames == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>]),
</code><code id="line-45" aria-labelledby="45">               <span class="str">"</span><span class="str">method2 should have the right exposure global names</span><span class="str">"</span>)
</code><code id="line-46" aria-labelledby="46">
</code><code id="line-47" aria-labelledby="47">    harness.ok(iface.exposureSet == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>, <span class="str">"</span><span class="str">Bar</span><span class="str">"</span>]),
</code><code id="line-48" aria-labelledby="48">               <span class="str">"</span><span class="str">Iface should have the right exposure set</span><span class="str">"</span>)
</code><code id="line-49" aria-labelledby="49">    harness.ok(iface._exposureGlobalNames == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>, <span class="str">"</span><span class="str">Bar1</span><span class="str">"</span>]),
</code><code id="line-50" aria-labelledby="50">               <span class="str">"</span><span class="str">Iface should have the right exposure global names</span><span class="str">"</span>)
</code><code id="line-51" aria-labelledby="51">
</code><code id="line-52" aria-labelledby="52">    parser = parser.reset()
</code><code id="line-53" aria-labelledby="53">    parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-54" aria-labelledby="54"><span class="str">      [PrimaryGlobal] interface Foo {};</span><span class="str">
</span></code><code id="line-55" aria-labelledby="55"><span class="str">      [Global=(Bar1,Bar2)] interface Bar {};</span><span class="str">
</span></code><code id="line-56" aria-labelledby="56"><span class="str">      [Global=Baz2] interface Baz {};</span><span class="str">
</span></code><code id="line-57" aria-labelledby="57"><span class="str">
</span></code><code id="line-58" aria-labelledby="58"><span class="str">      interface Iface2 {</span><span class="str">
</span></code><code id="line-59" aria-labelledby="59"><span class="str">        void method3();</span><span class="str">
</span></code><code id="line-60" aria-labelledby="60"><span class="str">      };</span><span class="str">
</span></code><code id="line-61" aria-labelledby="61"><span class="str">    </span><span class="str">"""</span>)
</code><code id="line-62" aria-labelledby="62">    results = parser.finish()
</code><code id="line-63" aria-labelledby="63">
</code><code id="line-64" aria-labelledby="64">    harness.check(len(results), 4, <span class="str">"</span><span class="str">Should know about four things</span><span class="str">"</span>);
</code><code id="line-65" aria-labelledby="65">    iface = results[3]
</code><code id="line-66" aria-labelledby="66">    harness.ok(isinstance(iface, WebIDL.IDLInterface),
</code><code id="line-67" aria-labelledby="67">               <span class="str">"</span><span class="str">Should have an interface here</span><span class="str">"</span>);
</code><code id="line-68" aria-labelledby="68">    members = iface.members
</code><code id="line-69" aria-labelledby="69">    harness.check(len(members), 1, <span class="str">"</span><span class="str">Should have one member</span><span class="str">"</span>)
</code><code id="line-70" aria-labelledby="70">
</code><code id="line-71" aria-labelledby="71">    harness.ok(members[0].exposureSet == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>]),
</code><code id="line-72" aria-labelledby="72">               <span class="str">"</span><span class="str">method3 should have the right exposure set</span><span class="str">"</span>)
</code><code id="line-73" aria-labelledby="73">    harness.ok(members[0]._exposureGlobalNames == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>]),
</code><code id="line-74" aria-labelledby="74">               <span class="str">"</span><span class="str">method3 should have the right exposure global names</span><span class="str">"</span>)
</code><code id="line-75" aria-labelledby="75">
</code><code id="line-76" aria-labelledby="76">    harness.ok(iface.exposureSet == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>]),
</code><code id="line-77" aria-labelledby="77">               <span class="str">"</span><span class="str">Iface2 should have the right exposure set</span><span class="str">"</span>)
</code><code id="line-78" aria-labelledby="78">    harness.ok(iface._exposureGlobalNames == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>]),
</code><code id="line-79" aria-labelledby="79">               <span class="str">"</span><span class="str">Iface2 should have the right exposure global names</span><span class="str">"</span>)
</code><code id="line-80" aria-labelledby="80">
</code><code id="line-81" aria-labelledby="81">    parser = parser.reset()
</code><code id="line-82" aria-labelledby="82">    parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-83" aria-labelledby="83"><span class="str">      [PrimaryGlobal] interface Foo {};</span><span class="str">
</span></code><code id="line-84" aria-labelledby="84"><span class="str">      [Global=(Bar1,Bar2)] interface Bar {};</span><span class="str">
</span></code><code id="line-85" aria-labelledby="85"><span class="str">      [Global=Baz2] interface Baz {};</span><span class="str">
</span></code><code id="line-86" aria-labelledby="86"><span class="str">
</span></code><code id="line-87" aria-labelledby="87"><span class="str">      [Exposed=Foo]</span><span class="str">
</span></code><code id="line-88" aria-labelledby="88"><span class="str">      interface Iface3 {</span><span class="str">
</span></code><code id="line-89" aria-labelledby="89"><span class="str">        void method4();</span><span class="str">
</span></code><code id="line-90" aria-labelledby="90"><span class="str">      };</span><span class="str">
</span></code><code id="line-91" aria-labelledby="91"><span class="str">
</span></code><code id="line-92" aria-labelledby="92"><span class="str">      [Exposed=(Foo,Bar1)]</span><span class="str">
</span></code><code id="line-93" aria-labelledby="93"><span class="str">      interface Mixin {</span><span class="str">
</span></code><code id="line-94" aria-labelledby="94"><span class="str">        void method5();</span><span class="str">
</span></code><code id="line-95" aria-labelledby="95"><span class="str">      };</span><span class="str">
</span></code><code id="line-96" aria-labelledby="96"><span class="str">
</span></code><code id="line-97" aria-labelledby="97"><span class="str">      Iface3 implements Mixin;</span><span class="str">
</span></code><code id="line-98" aria-labelledby="98"><span class="str">    </span><span class="str">"""</span>)
</code><code id="line-99" aria-labelledby="99">    results = parser.finish()
</code><code id="line-100" aria-labelledby="100">    harness.check(len(results), 6, <span class="str">"</span><span class="str">Should know about six things</span><span class="str">"</span>);
</code><code id="line-101" aria-labelledby="101">    iface = results[3]
</code><code id="line-102" aria-labelledby="102">    harness.ok(isinstance(iface, WebIDL.IDLInterface),
</code><code id="line-103" aria-labelledby="103">               <span class="str">"</span><span class="str">Should have an interface here</span><span class="str">"</span>);
</code><code id="line-104" aria-labelledby="104">    members = iface.members
</code><code id="line-105" aria-labelledby="105">    harness.check(len(members), 2, <span class="str">"</span><span class="str">Should have two members</span><span class="str">"</span>)
</code><code id="line-106" aria-labelledby="106">
</code><code id="line-107" aria-labelledby="107">    harness.ok(members[0].exposureSet == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>]),
</code><code id="line-108" aria-labelledby="108">               <span class="str">"</span><span class="str">method4 should have the right exposure set</span><span class="str">"</span>)
</code><code id="line-109" aria-labelledby="109">    harness.ok(members[0]._exposureGlobalNames == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>]),
</code><code id="line-110" aria-labelledby="110">               <span class="str">"</span><span class="str">method4 should have the right exposure global names</span><span class="str">"</span>)
</code><code id="line-111" aria-labelledby="111">
</code><code id="line-112" aria-labelledby="112">    harness.ok(members[1].exposureSet == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>, <span class="str">"</span><span class="str">Bar</span><span class="str">"</span>]),
</code><code id="line-113" aria-labelledby="113">               <span class="str">"</span><span class="str">method5 should have the right exposure set</span><span class="str">"</span>)
</code><code id="line-114" aria-labelledby="114">    harness.ok(members[1]._exposureGlobalNames == set([<span class="str">"</span><span class="str">Foo</span><span class="str">"</span>, <span class="str">"</span><span class="str">Bar1</span><span class="str">"</span>]),
</code><code id="line-115" aria-labelledby="115">               <span class="str">"</span><span class="str">method5 should have the right exposure global names</span><span class="str">"</span>)
</code><code id="line-116" aria-labelledby="116">
</code><code id="line-117" aria-labelledby="117">    parser = parser.reset()
</code><code id="line-118" aria-labelledby="118">    threw = False
</code><code id="line-119" aria-labelledby="119">    <span class="k">try</span>:
</code><code id="line-120" aria-labelledby="120">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-121" aria-labelledby="121"><span class="str">            [Exposed=Foo]</span><span class="str">
</span></code><code id="line-122" aria-labelledby="122"><span class="str">            interface Bar {</span><span class="str">
</span></code><code id="line-123" aria-labelledby="123"><span class="str">            };</span><span class="str">
</span></code><code id="line-124" aria-labelledby="124"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-125" aria-labelledby="125">
</code><code id="line-126" aria-labelledby="126">        results = parser.finish()
</code><code id="line-127" aria-labelledby="127">    <span class="k">except</span> Exception,x:
</code><code id="line-128" aria-labelledby="128">        threw = True
</code><code id="line-129" aria-labelledby="129">
</code><code id="line-130" aria-labelledby="130">    harness.ok(threw, <span class="str">"</span><span class="str">Should have thrown on invalid Exposed value on interface.</span><span class="str">"</span>)
</code><code id="line-131" aria-labelledby="131">
</code><code id="line-132" aria-labelledby="132">    parser = parser.reset()
</code><code id="line-133" aria-labelledby="133">    threw = False
</code><code id="line-134" aria-labelledby="134">    <span class="k">try</span>:
</code><code id="line-135" aria-labelledby="135">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-136" aria-labelledby="136"><span class="str">            interface Bar {</span><span class="str">
</span></code><code id="line-137" aria-labelledby="137"><span class="str">              [Exposed=Foo]</span><span class="str">
</span></code><code id="line-138" aria-labelledby="138"><span class="str">              readonly attribute bool attr;</span><span class="str">
</span></code><code id="line-139" aria-labelledby="139"><span class="str">            };</span><span class="str">
</span></code><code id="line-140" aria-labelledby="140"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-141" aria-labelledby="141">
</code><code id="line-142" aria-labelledby="142">        results = parser.finish()
</code><code id="line-143" aria-labelledby="143">    <span class="k">except</span> Exception,x:
</code><code id="line-144" aria-labelledby="144">        threw = True
</code><code id="line-145" aria-labelledby="145">
</code><code id="line-146" aria-labelledby="146">    harness.ok(threw, <span class="str">"</span><span class="str">Should have thrown on invalid Exposed value on attribute.</span><span class="str">"</span>)
</code><code id="line-147" aria-labelledby="147">
</code><code id="line-148" aria-labelledby="148">    parser = parser.reset()
</code><code id="line-149" aria-labelledby="149">    threw = False
</code><code id="line-150" aria-labelledby="150">    <span class="k">try</span>:
</code><code id="line-151" aria-labelledby="151">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-152" aria-labelledby="152"><span class="str">            interface Bar {</span><span class="str">
</span></code><code id="line-153" aria-labelledby="153"><span class="str">              [Exposed=Foo]</span><span class="str">
</span></code><code id="line-154" aria-labelledby="154"><span class="str">              void operation();</span><span class="str">
</span></code><code id="line-155" aria-labelledby="155"><span class="str">            };</span><span class="str">
</span></code><code id="line-156" aria-labelledby="156"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-157" aria-labelledby="157">
</code><code id="line-158" aria-labelledby="158">        results = parser.finish()
</code><code id="line-159" aria-labelledby="159">    <span class="k">except</span> Exception,x:
</code><code id="line-160" aria-labelledby="160">        threw = True
</code><code id="line-161" aria-labelledby="161">
</code><code id="line-162" aria-labelledby="162">    harness.ok(threw, <span class="str">"</span><span class="str">Should have thrown on invalid Exposed value on operation.</span><span class="str">"</span>)
</code><code id="line-163" aria-labelledby="163">
</code><code id="line-164" aria-labelledby="164">    parser = parser.reset()
</code><code id="line-165" aria-labelledby="165">    threw = False
</code><code id="line-166" aria-labelledby="166">    <span class="k">try</span>:
</code><code id="line-167" aria-labelledby="167">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-168" aria-labelledby="168"><span class="str">            interface Bar {</span><span class="str">
</span></code><code id="line-169" aria-labelledby="169"><span class="str">              [Exposed=Foo]</span><span class="str">
</span></code><code id="line-170" aria-labelledby="170"><span class="str">              const long constant = 5;</span><span class="str">
</span></code><code id="line-171" aria-labelledby="171"><span class="str">            };</span><span class="str">
</span></code><code id="line-172" aria-labelledby="172"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-173" aria-labelledby="173">
</code><code id="line-174" aria-labelledby="174">        results = parser.finish()
</code><code id="line-175" aria-labelledby="175">    <span class="k">except</span> Exception,x:
</code><code id="line-176" aria-labelledby="176">        threw = True
</code><code id="line-177" aria-labelledby="177">
</code><code id="line-178" aria-labelledby="178">    harness.ok(threw, <span class="str">"</span><span class="str">Should have thrown on invalid Exposed value on constant.</span><span class="str">"</span>)
</code><code id="line-179" aria-labelledby="179">
</code><code id="line-180" aria-labelledby="180">    parser = parser.reset()
</code><code id="line-181" aria-labelledby="181">    threw = False
</code><code id="line-182" aria-labelledby="182">    <span class="k">try</span>:
</code><code id="line-183" aria-labelledby="183">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-184" aria-labelledby="184"><span class="str">            [Global] interface Foo {};</span><span class="str">
</span></code><code id="line-185" aria-labelledby="185"><span class="str">            [Global] interface Bar {};</span><span class="str">
</span></code><code id="line-186" aria-labelledby="186"><span class="str">
</span></code><code id="line-187" aria-labelledby="187"><span class="str">            [Exposed=Foo]</span><span class="str">
</span></code><code id="line-188" aria-labelledby="188"><span class="str">            interface Baz {</span><span class="str">
</span></code><code id="line-189" aria-labelledby="189"><span class="str">              [Exposed=Bar]</span><span class="str">
</span></code><code id="line-190" aria-labelledby="190"><span class="str">              void method();</span><span class="str">
</span></code><code id="line-191" aria-labelledby="191"><span class="str">            };</span><span class="str">
</span></code><code id="line-192" aria-labelledby="192"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-193" aria-labelledby="193">
</code><code id="line-194" aria-labelledby="194">        results = parser.finish()
</code><code id="line-195" aria-labelledby="195">    <span class="k">except</span> Exception,x:
</code><code id="line-196" aria-labelledby="196">        threw = True
</code><code id="line-197" aria-labelledby="197">
</code><code id="line-198" aria-labelledby="198">    harness.ok(threw, <span class="str">"</span><span class="str">Should have thrown on member exposed where its interface is not.</span><span class="str">"</span>)
</code><code id="line-199" aria-labelledby="199">
</code><code id="line-200" aria-labelledby="200">    parser = parser.reset()
</code><code id="line-201" aria-labelledby="201">    threw = False
</code><code id="line-202" aria-labelledby="202">    <span class="k">try</span>:
</code><code id="line-203" aria-labelledby="203">        parser.parse(<span class="str">"""</span><span class="str">
</span></code><code id="line-204" aria-labelledby="204"><span class="str">            [Global] interface Foo {};</span><span class="str">
</span></code><code id="line-205" aria-labelledby="205"><span class="str">            [Global] interface Bar {};</span><span class="str">
</span></code><code id="line-206" aria-labelledby="206"><span class="str">
</span></code><code id="line-207" aria-labelledby="207"><span class="str">            [Exposed=Foo]</span><span class="str">
</span></code><code id="line-208" aria-labelledby="208"><span class="str">            interface Baz {</span><span class="str">
</span></code><code id="line-209" aria-labelledby="209"><span class="str">              void method();</span><span class="str">
</span></code><code id="line-210" aria-labelledby="210"><span class="str">            };</span><span class="str">
</span></code><code id="line-211" aria-labelledby="211"><span class="str">
</span></code><code id="line-212" aria-labelledby="212"><span class="str">            [Exposed=Bar]</span><span class="str">
</span></code><code id="line-213" aria-labelledby="213"><span class="str">            interface Mixin {};</span><span class="str">
</span></code><code id="line-214" aria-labelledby="214"><span class="str">
</span></code><code id="line-215" aria-labelledby="215"><span class="str">            Baz implements Mixin;</span><span class="str">
</span></code><code id="line-216" aria-labelledby="216"><span class="str">        </span><span class="str">"""</span>)
</code><code id="line-217" aria-labelledby="217">
</code><code id="line-218" aria-labelledby="218">        results = parser.finish()
</code><code id="line-219" aria-labelledby="219">    <span class="k">except</span> Exception,x:
</code><code id="line-220" aria-labelledby="220">        threw = True
</code><code id="line-221" aria-labelledby="221">
</code><code id="line-222" aria-labelledby="222">    harness.ok(threw, <span class="str">"</span><span class="str">Should have thrown on LHS of implements being exposed where RHS is not.</span><span class="str">"</span>)
</code></pre>
        </td>
      </tr>
    </tbody>
  </table>

    </div>

    
      
        <div id="foot" class="footer">
          This page was generated by DXR
          <span class="pretty-date" data-datetime="Tue, 08 Mar 2016 11:22:17 +0000"></span>.
        </div>
      
    

    

    <!-- avoid inline JS and use data attributes instead. Hackey but hey... -->
    <span id="data" data-root="" data-search="/mozilla-central/search" data-tree="mozilla-central"></span>
    <span id="state" data-offset="0" data-limit="100" data-results-line-count="" data-eof="False"></span>

    
  
      <script src="/static/js/libs/jquery203.0a6e846b.js"></script>
      <script src="/static/js/libs/nunjucks-slim.43040a7a.min.js"></script>
      <script src="/static/js/templates.58d6208e.js"></script>
      <script src="/static/js/utils.994d6cf1.js"></script>
      <script src="/static/js/dxr.d4939bfc.js"></script>
      <script src="/static/js/context_menu.8b4315f5.js"></script>
      <script src="/static/js/filter.fd0341f1.js"></script>
    
  <script src="/static/js/panel.222eee37.js"></script>
  <script src="/static/js/tree-selector.58293846.js"></script>
  <script src="/static/js/code-highlighter.9d7636ad.js"></script>


    

  </body>
</html>