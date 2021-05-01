#!/bin/bash
#
# This script generates the README.md file based on the rustdoc documentation of the crate.
#
# Note that this currently only supports the following types of links:
# - [StructName]
# - [StructName::method_name]
# other types (e.g. namespaces, different crates, ...) are not supported
#
set -e

# First, use "cargo readme" to generate the README.md file
cargo readme > README.md

# But "cargo readme" doesn't support links, so we need to fix them

# Replace [MyStructName] with [MyStructName](https://docs.rs/binary-layout/latest/binary_layout/struct.MyStructName.html)
sed -i 's|\[\([a-zA-Z_]\+\)\]\([^(]\)|[\1](https://docs.rs/binary-layout/latest/binary_layout/struct.\1.html)\2|g' README.md

# Replace [my_macro_name!] with [my_macro_name!](https://docs.rs/binary-layout/latest/binary_layout/macro.my_macro_name.html)
sed -i 's|\[\([a-zA-Z_]\+\)!\]\([^(]\)|[\1!](https://docs.rs/binary-layout/latest/binary_layout/macro.\1.html)\2|g' README.md

# Replace [MyStructName::my_method_name] with [MyStructName::my_method_name](https://docs.rs/binary-layout/latest/binary_layout/struct.MyStructName.html#method.my_method_name)
sed -i 's|\[\([a-zA-Z_]\+\)::\([a-zA-Z_]\+\)\]\([^(]\)|[\1::\2](https://docs.rs/binary-layout/latest/binary_layout/struct.\1.html#method.\2)\3|g' README.md

echo Success
