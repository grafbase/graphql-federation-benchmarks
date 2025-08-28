#!/usr/bin/env bash

set -euo pipefail
project_dir="$(dirname "$(dirname "$(dirname "$(readlink -f "$0")")")")"
cd "$project_dir/benchmarks/$1/cosmo"

wgc router compose -i compose.yml -o supergraph.json
