#!/usr/bin/env bash

set -e

ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && cd .. && pwd )"

pushd "$ROOT" &> /dev/null

  TAG=`git rev-parse --short HEAD`
  CI_COMMIT=${TAG} cargo build

popd &> /dev/null