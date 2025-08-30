#!/usr/bin/env bash

set -euxo pipefail

docker stop $(docker ps -a -q) -t 2 && docker rm $(docker ps -a -q)
