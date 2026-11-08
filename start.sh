#!/bin/sh
cd "$(dirname "$0")/sprite-editor" || exit 1
if [ ! -d node_modules ]; then
  if ! command -v node >/dev/null 2>&1; then
    echo "Node.js is required. Install it from https://nodejs.org and run this again."
    exit 1
  fi
  npm install || exit 1
fi
exec npm start
