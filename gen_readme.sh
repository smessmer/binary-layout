#!/bin/bash
#
# This script generates the README.md file based on the rustdoc documentation of the crate.
#
# Note that this currently only supports the following types of links:
# - [struct@StructName]
# - [FieldCopyAccess::method_name]
# - [FieldSliceAccess::method_name]
# - [FieldView::method_name]
# - [trait@TraitName]
# other types (e.g. namespaces, different crates, ...) are not supported
#
set -e

# First, use "cargo readme" to generate the README.md file
cargo readme > README.md

# But "cargo readme" doesn't support links, so we need to fix them

# Replace [struct@MyStructName] with [MyStructName](https://docs.rs/binary-layout/latest/binary_layout/struct.MyStructName.html)
sed -i 's|\[struct@\([a-zA-Z_]\+\)\]\([^(]\)|[\1](https://docs.rs/binary-layout/latest/binary_layout/struct.\1.html)\2|g' README.md

# Replace [trait@MyStructName] with [MyStructName](https://docs.rs/binary-layout/latest/binary_layout/struct.MyStructName.html)
sed -i 's|\[trait@\([a-zA-Z_]\+\)\]\([^(]\)|[\1](https://docs.rs/binary-layout/latest/binary_layout/trait.\1.html)\2|g' README.md

# Replace [my_macro_name!] with [my_macro_name!](https://docs.rs/binary-layout/latest/binary_layout/macro.my_macro_name.html)
sed -i 's|\[\([a-zA-Z_]\+\)!\]\([^(]\)|[\1!](https://docs.rs/binary-layout/latest/binary_layout/macro.\1.html)\2|g' README.md

# Replace [FieldView::my_method_name] with [FieldView::my_method_name](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.my_method_name)
sed -i 's|\[FieldView::\([a-zA-Z_]\+\)\]\([^(]\)|[FieldView::\1](https://docs.rs/binary-layout/latest/binary_layout/struct.FieldView.html#method.\1)\2|g' README.md

# Replace [FieldCopyAccess::my_method_name] with [FieldCopyAccess::my_method_name](https://docs.rs/binary-layout/latest/binary_layout/trait.FieldCopyAccess.html#tymethod.my_method_name)
sed -i 's|\[FieldCopyAccess::\([a-zA-Z_]\+\)\]\([^(]\)|[FieldCopyAccess::\1](https://docs.rs/binary-layout/latest/binary_layout/trait.FieldCopyAccess.html#tymethod.\1)\2|g' README.md

# Replace [FieldSliceAccess::my_method_name] with [FieldSliceAccess::my_method_name](https://docs.rs/binary-layout/latest/binary_layout/trait.FieldSliceAccess.html#tymethod.my_method_name)
sed -i 's|\[FieldSliceAccess::\([a-zA-Z_]\+\)\]\([^(]\)|[FieldSliceAccess::\1](https://docs.rs/binary-layout/latest/binary_layout/trait.FieldSliceAccess.html#tymethod.\1)\2|g' README.md

# Replace [...](crate::example::icmp_packet) with [...](https://docs.rs/binary-layout/latest/binary_layout/example/icmp_packet/index.html)
sed -i 's|\[\([a-zA-Z_]\+\)\](crate::example::icmp_packet)|[\1](https://docs.rs/binary-layout/latest/binary_layout/example/icmp_packet/index.html)|g' README.md

echo Success
