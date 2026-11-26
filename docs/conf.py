# Sphinx configuration of the Guaraci documentation.
#
# English is the source language; Portuguese and Spanish are gettext
# translations in locale/. Each language is built separately, e.g.
#   sphinx-build -D language=pt_BR docs _build/pt-br
# Published on Read the Docs (.readthedocs.yaml): one project per language,
# linked as translations of the English project.

import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "_ext"))

project = "Guaraci"
author = "Harlley Hauradou"
copyright = "Guaraci"
release = "0.1"
language = "en"

# Translations (gettext): docs/locale/<lang>/LC_MESSAGES/*.po
locale_dirs = ["locale/"]
gettext_compact = False

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
html_favicon = "_static/brand/favicon.svg"
html_static_path = ["_static"]
templates_path = ["_templates"]
html_css_files = [
    # IBM Plex (SIL Open Font License), the typeface of the Guaraci brand
    "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans+Condensed:wght@500;600&family=IBM+Plex+Sans:ital,wght@0,400;0,500;0,600;1,400&display=swap",
    "custom.css",
]
html_show_sourcelink = False

html_theme_options = {
    "logo": {
        "image_light": "_static/brand/guaraci-assinatura.svg",
        "image_dark": "_static/brand/guaraci-assinatura-sobre-escuro.svg",
        "alt_text": "guaraci",
    },
    "github_url": "https://github.com/HarlleyHauradou/guaraci",
    "header_links_before_dropdown": 6,
    "navbar_align": "left",
    "navbar_end": ["language-switcher", "theme-switcher", "navbar-icon-links"],
    "secondary_sidebar_items": ["page-toc", "edit-this-page"],
    "use_edit_page_button": True,
    "show_toc_level": 2,
    "navigation_with_keys": True,
    "show_prev_next": True,
    "footer_start": ["copyright"],
    "footer_end": ["theme-version"],
    "pygments_light_style": "default",
    "pygments_dark_style": "github-dark",
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


def setup(app):
    from gci_lexer import GciLexer

    app.add_lexer("gci", GciLexer)
