# Little Tomato Agent Guide

Before changing this repository, read:

1. D:\Tomato\LTT\00-GPT-6交接说明.md
2. D:\Tomato\LTT\需求文档.md
3. D:\Tomato\LTT\技术路线.md
4. D:\Tomato\LTT\开发计划.md
5. D:\Tomato\LTT\测试与验收.md

The product decisions in those documents are already confirmed. Do not restart the discovery interview.

## Mandatory local environment rule

Do not install SDKs, package caches, dependencies, temporary build files, runtime test data, or compiler outputs on C:. Use D:\Coding.

Important paths:

- CARGO_HOME: D:\Coding\Rust\cargo
- RUSTUP_HOME: D:\Coding\Rust\rustup
- CARGO_TARGET_DIR: D:\Coding\Build\Cargo
- npm cache: D:\Coding\npm-cache
- npm global prefix: D:\Coding\npm-global
- node_modules target: D:\Coding\Packages\LittleTomato\node_modules
- TEMP/TMP: D:\Coding\Temp
- LITTLE_TOMATO_RUNTIME_DIR: D:\Coding\Runtime\LittleTomato

The existing Visual Studio Build Tools installation at D:\Microsoft Visual Studio\18\BuildTools predates this project. Use it, but do not move, modify, or uninstall it without explicit user approval.

## Working rules

- Inspect the running UI visually and iterate on it.
- Keep all user data local by default.
- Keep SQL and filesystem access in Rust.
- Preserve uncommitted user and agent changes.
- Do not commit or push unless explicitly requested.
- Run TypeScript, Rust formatting, Clippy, and relevant integration checks before handoff.
- Keep the app runnable after each development stage.
