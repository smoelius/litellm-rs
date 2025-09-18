# Docker Build for LiteLLM-RS

This directory contains Docker build files and scripts for the LiteLLM-RS gateway.

## Build Issues and Solutions

### CMAKE Missing Error

If you encounter the following error during Docker builds:

```
Missing dependency: cmake
thread 'main' panicked at .../aws-lc-sys-0.31.0/builder/main.rs:463:40:
called `Result::unwrap()` on an `Err` value: "Required build dependency is missing. Halting build."
```

This happens because the `aws-lc-sys` crate (used by `rustls` for TLS support) requires cmake and other build tools for compilation.

### Solution

The Dockerfile has been updated to include all necessary build dependencies:

- `cmake` - Required by aws-lc-sys
- `build-essential` - C/C++ compilation tools
- `clang` and `llvm` - Modern C/C++ compiler
- `gcc-arm-linux-gnueabihf` - ARM cross-compilation support
- `libc6-dev-armhf-cross` - ARM development headers

### Environment Variables

The following environment variables are set for proper compilation:

```bash
CMAKE=cmake
CC=clang
CXX=clang++
CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
PKG_CONFIG_ALLOW_CROSS=1
```

### Files

- `Dockerfile` - Main multi-stage Dockerfile for all architectures
- `Dockerfile.arm` - ARM-specific Dockerfile with enhanced dependencies
- `build.sh` - Build script with error handling and cleanup
- `.dockerignore` - Files to exclude from Docker context

### Usage

```bash
# Build for current architecture
./deployment/docker/build.sh

# Build with custom tag
./deployment/docker/build.sh -t v1.0.0

# Use ARM-specific Dockerfile
docker build -f deployment/docker/Dockerfile.arm -t litellm-rs:arm .
```

### Dependency Chain

The cmake requirement comes from this dependency chain:

```
litellm-rs -> rustls -> aws-lc-rs -> aws-lc-sys (requires cmake)
```

This is a common pattern in Rust applications that use TLS/SSL functionality.

### Troubleshooting

1. **Build fails on ARM platforms**: Use `Dockerfile.arm` which includes additional ARM cross-compilation tools
2. **Permission errors**: Ensure Docker has proper permissions and the daemon is running
3. **Out of space**: The build process can be large, ensure sufficient disk space (2GB+ recommended)
4. **Network issues**: Some dependencies are downloaded during build, ensure internet connectivity

### GitHub Actions

The CI/CD pipeline automatically builds for multiple architectures including:
- `linux/amd64`
- `linux/arm64`
- `linux/arm/v7`

The enhanced Dockerfile should resolve cmake-related build failures across all platforms.