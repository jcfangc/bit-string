mod chunk_eq;
mod leading;
#[cfg(all(
    not(feature = "compile-time-dispatch"),
    any(target_arch = "x86", target_arch = "x86_64")
))]
mod runtime;
mod trailing;

pub(crate) use leading::leading;
pub(crate) use trailing::trailing;
