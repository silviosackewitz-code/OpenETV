#!/bin/bash
# Change into the directory this script lives in (works even if the folder
# is moved or renamed).
cd "$(dirname "$0")"

.venv/bin/python -m streamlit run app.py
