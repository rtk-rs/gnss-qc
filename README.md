GNSS Quality Control
====================

[![Rust](https://github.com/rtk-rs/gnss-qc/actions/workflows/rust.yml/badge.svg)](https://github.com/rtk-rs/gnss-qc/actions/workflows/rust.yml)
[![Rust](https://github.com/rtk-rs/gnss-qc/actions/workflows/daily.yml/badge.svg)](https://github.com/rtk-rs/gnss-qc/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/gnss-qc/badge.svg)](https://docs.rs/gnss-qc/)
[![crates.io](https://img.shields.io/crates/d/gnss-qc.svg)](https://crates.io/crates/gnss-qc)

[![MRSV](https://img.shields.io/badge/MSRV-1.81.0-orange?style=for-the-badge)](https://github.com/rust-lang/rust/releases/tag/1.81.0)
[![License](https://img.shields.io/badge/license-MPL_2.0-orange?style=for-the-badge&logo=mozilla)](https://github.com/rtk-rs/qc-traits/blob/main/LICENSE)

The GNSS Quality Control (QC) library is an advanced library that proposes
from basic to advanced GNSS and Geodesy processing pipelines.

It is made possible by the complex combination of several frameworks and libraries.
It is important to understand this library's features & options.

This library is part of the [RTK-rs framework](https://github.com/rtk-rs) which
is delivered under the [Mozilla V2 Public](https://www.mozilla.org/en-US/MPL/2.0) license.

## Core level

The fundammental blocks that we rely upon, at all times

- [Hifitime by Nyx-Space](https://github.com/nyx-space/hifitime) 
that provides Epoch and TimeScale definitions
- [GNSS by RTK-rs](https://github.com/rtk-rs/qc-traits) that provides
Constellation and SV definitions
- [Qc Traits by RTK-rs](https://github.com/rtk-rs/qc-traits) that provides 
shared behavior by all GNSS libraries
- [The RINEX library by RTK-rs](https://github.com/rtk-rs/rinex) because we consider
the RINEX files as the most fundamental. It is currently not possible to build
this library without RINEX support (say: SP3 only application). But that could easily be changed.

## Basic and default features

- `flate2` is activated by default, and allows Gzip compressed files to be naturally supported.
- `sp3` is activated by default, because we consider people interested in GNSS post processing
are interested in high precision at all times. This is easily changed by de-activating this crate feature.

## Navigation feature

`nav` is the most advanced feature. It allows post processed navigation and is the heaviest option.
This option relies on [ANISE by Nyx-Space](https://github.com/nyx-space/anise).

If you are only interested in file processing and management, you should not activate Post Processed navigation support.

## Deploying without navigation support

Without navigation support, this library will allow GNSS context creation and basic processing.
You will not access the most advanced solvers.
