#!/bin/bash

diesel migration run
cargo test --tests
