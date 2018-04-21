#! /bin/bash

cargo build && sudo target/debug/game-kb-driver $*
