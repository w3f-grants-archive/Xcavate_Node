#!/usr/bin/env bash

# This file is part of Substrate.
# Copyright (C) 2022 Parity Technologies (UK) Ltd.
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# This script has three parts which all use the Substrate runtime:
# - Pallet benchmarking to update the pallet weights
# - Overhead benchmarking for the Extrinsic and Block weights
# - Machine benchmarking
#
# Should be run on a reference machine to gain accurate benchmarks
# current reference machine: https://github.com/paritytech/substrate/pull/5848

standard_args=" --release --features=runtime-benchmarks"

pallets=(
	pallet-sudo
)

cargo build $standard_args

for pallet in "${pallets[@]}"; do
    ./target/release/node-template benchmark pallet \
        --chain dev \
        --wasm-execution=compiled \
        --pallet "$pallet" \
        --no-storage-info \
        --no-median-slopes \
        --no-min-squares \
        --extrinsic="*" \
        --heap-pages=4096 \
        --steps 50 \
        --repeat 20 \
		--output="./runtime/src/substrate_weights/${pallet//-/_}.rs"
done