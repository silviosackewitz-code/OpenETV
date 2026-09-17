"""Entry point of the built app.

PyInstaller runs this script as the top level — hence the absolute import
instead of the relative ones of `etvlib/__main__.py`. Without arguments the
window opens; the arguments are those of `python -m etvlib`.
"""

import sys

from etvlib.cli import main

if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
