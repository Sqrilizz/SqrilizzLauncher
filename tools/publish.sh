#!/usr/bin/env bash
set -e

# sqrilizz_shared
cd crates/shared
cargo publish
cd ..

# sqrilizz_net
cd net
cargo publish
cd ..

# sqrilizz_auth
cd auth
cargo publish
cd ..

# sqrilizz_core
cd core
cargo publish
cd ..

# sqrilizz_mods
cd mods
cargo publish
cd ..

# sqrilizz_parse
cd parse
cargo publish
cd ..

# sqrilizz_pkg
cd pkg
cargo publish
cd ..

# sqrilizz_pkg_gen
cd pkg_gen
cargo publish
cd ..

# sqrilizz_instance
cd instance
cargo publish
cd ..

# sqrilizz_config
cd config
cargo publish
cd ..

# sqrilizz_options
cd options
cargo publish
cd ..

# sqrilizz
cd ..
cargo publish

# sqrilizz_cli
cd crates/cli
cargo publish
cd ../..
