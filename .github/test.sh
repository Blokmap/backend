#!/bin/bash

diesel migration run
CI=true cargo test --tests
