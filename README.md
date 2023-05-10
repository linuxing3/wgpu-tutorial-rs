# wgpu tutorial rs 


## wgpu

### device and queue

### surface

### pipeline

#### vertex buffer and index buffer

### image and texture

### camera and uniform buffer

## winit

### window

### loop

## Imgui

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Yatekii/imgui-wgpu-rs/build.yml?branch=master)
[![Documentation](https://docs.rs/imgui-wgpu/badge.svg)](https://docs.rs/imgui-wgpu)
[![Crates.io](https://img.shields.io/crates/v/imgui-wgpu)](https://crates.io/crates/imgui-wgpu)
![License](https://img.shields.io/crates/l/imgui-wgpu)

Draw dear imgui UIs as a wgpu render pass. Based on [imgui-gfx-renderer](https://github.com/Gekkio/imgui-rs/tree/master/imgui-gfx-renderer) from [imgui-rs](https://github.com/Gekkio/imgui-rs).


### Usage

For usage, please have a look at the [example](examples/hello-world.rs).

### Status

Supports `wgpu` `0.14` and imgui `0.9`. `winit-0.27` is used with the examples.

Contributions are very welcome.

# Troubleshooting

## Cargo resolver

Starting with [`wgpu` 0.10](https://github.com/gfx-rs/wgpu/blob/06316c1bac8b78ac04d762cfb1a886bd1d453b30/CHANGELOG.md#v010-2021-08-18), the [resolver version](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions) needs to be set in your `Cargo.toml` to avoid build errors:

```toml
resolver = "2"
```
