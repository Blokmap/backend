#!/usr/bin/env bash

# Credit: https://github.com/snok/install-poetry/blob/main/main.sh

set -eo pipefail

download_script() {
  python3 -c 'import urllib.request, sys; print(urllib.request.urlopen(f"{sys.argv[1]}").read().decode("utf8"))' $1
}

INSTALL_PATH="${POETRY_HOME:-$HOME/.local}"

INSTALLATION_SCRIPT="$(mktemp)"
download_script "https://install.python-poetry.org" > "$INSTALLATION_SCRIPT"

POETRY_HOME=$INSTALL_PATH python3 "$INSTALLATION_SCRIPT" --yes

echo "$INSTALL_PATH/bin" >>"$GITHUB_PATH"
export PATH="$INSTALL_PATH/bin:$PATH"

poetry config virtualenvs.create true
poetry config virtualenvs.in-project true

echo "VENV=.venv/bin/activate" >>"$GITHUB_ENV"
