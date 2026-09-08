# Sandlot examples

A showcase of the usage & capabilities of certain SDL features, and how Sandlot
wraps them in an intuitive interface.

# Important

GPU examples will not compile by default, since the repository only contains shader sources,
not their compiled IR. To fix that, run one of the following platform-specific scripts:

- `scripts/windows/compile-hlsl-shaders.ps1`
- `scripts/windows/compile-spirv-shaders.ps1`
- `scripts/macos/compile-metal-shaders.sh`
- `scripts/unix/compile-spirv-shaders.sh`
