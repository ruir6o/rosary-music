#!/usr/bin/env bash

cargo vendor third-party/vendor | tee .cargo/config.toml
