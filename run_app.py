"""
Entry point for the PyInstaller-built executable.

Streamlit apps are normally launched via the `streamlit run app.py` CLI
command. Inside a frozen executable there is no such CLI available, so this
script starts Streamlit's own CLI programmatically instead, pointing it at
the bundled copy of app.py.
"""
import os
import sys

from streamlit.web import cli as stcli


def resource_path(relative_path):
    """Resolve a path both when run normally and when frozen by PyInstaller
    (bundled files are extracted to sys._MEIPASS at runtime).
    """
    base_path = getattr(sys, "_MEIPASS", os.path.dirname(os.path.abspath(__file__)))
    return os.path.join(base_path, relative_path)


if __name__ == "__main__":
    sys.argv = [
        "streamlit",
        "run",
        resource_path("app.py"),
        "--global.developmentMode=false",
        "--server.headless=false",
    ]
    sys.exit(stcli.main())
