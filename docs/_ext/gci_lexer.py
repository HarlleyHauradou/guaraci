"""Pygments lexer for Guaraci input files (.gci).

Units get a token type of their own (Number.Unit, CSS class `m-Unit`) so that
the documentation can show physical quantities in the brand colour:
principle no. 1 ("input is written for humans") applied to syntax highlighting.
"""

from pygments.lexer import RegexLexer, bygroups
from pygments.token import Comment, Keyword, Name, Number, Operator, Punctuation, String, Whitespace

UNIT = r"[A-Za-zµμ°%][A-Za-z0-9µμ°%/*.^·]*(?:\^-?\d+)?"
NUMBER = r"[-+]?(?:\d[\d_]*\.?[\d_]*|\.\d+)(?:[eE][-+]?\d+)?"


class GciLexer(RegexLexer):
    name = "Guaraci input"
    aliases = ["gci", "guaraci"]
    filenames = ["*.gci"]

    tokens = {
        "root": [
            (r"[ \t]+", Whitespace),
            (r"\n", Whitespace),
            (r"#.*$", Comment.Single),
            # [section] or [section name]
            (r"^(\[)([A-Za-z_][\w-]*)(?:([ \t]+)([^\]\s#]+))?(\])",
             bygroups(Punctuation, Keyword.Declaration, Whitespace, Name.Class, Punctuation)),
            (r"^(include)\b", Keyword.Namespace),
            # key =
            (r"([A-Za-z_][\w-]*)([ \t]*)(=)", bygroups(Name.Attribute, Whitespace, Operator)),
            (r'"(?:\\.|[^"\\])*"', String.Double),
            # number with an optional unit: 10 cm, 10cm, 1.05 g/cm3, 89 %
            (r"(" + NUMBER + r")([ \t]*)(" + UNIT + r")", bygroups(Number, Whitespace, Number.Unit)),
            (NUMBER, Number),
            # unit after a list: [1, 2, 3] cm
            (r"(\])([ \t]*)(" + UNIT + r")(?=[ \t]*(?:#|$|,|\]|\}))", bygroups(Punctuation, Whitespace, Number.Unit)),
            (r"\b(?:on|off|yes|no|true|false)\b", Keyword.Constant),
            (r"[A-Za-z_][\w.\-/+]*", Name),
            (r"[\[\]{},]", Punctuation),
            (r".", Name),
        ]
    }
