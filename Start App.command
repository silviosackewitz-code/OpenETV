#!/bin/bash
# Change into the directory this script lives in (works even if the folder
# is moved or renamed).
cd "$(dirname "$0")"

source venv/bin/activate
streamlit run app.py
