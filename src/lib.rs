#[cfg(all(feature = "anchor", feature = "borsh-v1"))]
compile_error!(
    "Features `anchor` and `borsh-v1` are mutually exclusive. \
     Anchor uses borsh v0.10 internally and is incompatible with borsh v1."
);

pub mod types;
