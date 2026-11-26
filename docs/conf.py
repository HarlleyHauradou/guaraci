# Sphinx configuration of the Guaraci documentation (Read the Docs).

project = "Guaraci"
author = "Harlley Hauradou"
copyright = "Guaraci"
release = "0.1"
language = "pt_BR"

extensions = [
    "myst_parser",
    "sphinx_design",
    "sphinx_copybutton",
]

source_suffix = {".md": "markdown"}
master_doc = "index"
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

myst_enable_extensions = ["colon_fence", "deflist", "attrs_inline", "attrs_block", "substitution"]
myst_heading_anchors = 3

html_theme = "pydata_sphinx_theme"
html_title = "Guaraci"
html_logo = "_static/logo.svg"
html_favicon = "_static/logo.svg"
html_static_path = ["_static"]
html_css_files = ["custom.css"]
html_show_sourcelink = False

html_theme_options = {
    "logo": {"text": "Guaraci"},
    "github_url": "https://github.com/HarlleyHauradou/guaraci",
    "header_links_before_dropdown": 6,
    "navbar_align": "left",
    "navbar_end": ["theme-switcher", "navbar-icon-links"],
    "secondary_sidebar_items": ["page-toc", "edit-this-page"],
    "use_edit_page_button": True,
    "show_toc_level": 2,
    "navigation_with_keys": True,
    "show_prev_next": True,
    "footer_start": ["copyright"],
    "footer_end": ["theme-version"],
    "pygments_light_style": "tango",
    "pygments_dark_style": "monokai",
}

html_context = {
    "github_user": "HarlleyHauradou",
    "github_repo": "guaraci",
    "github_version": "main",
    "doc_path": "docs",
    "default_mode": "auto",
}

copybutton_prompt_text = r"\$ "
copybutton_prompt_is_regexp = True
