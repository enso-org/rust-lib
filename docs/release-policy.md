---
layout: developer-doc
title: Release Policy
category: distribution
tags: [distribution, release, release-policy, policy]
order: 3
---

# Release Policy

As an open-source project, it is incredibly important that we have a
well-defined release policy. This document defines said policy.

> **Once a release has been made it is immutable. The release should only ever
> be edited to mark it as broken. Nothing else should ever be changed.**

<!-- MarkdownTOC levels="2,3" autolink="true" -->

- [Versioning](#versioning)
- [Release Branches](#release-branches)
- [Release Workflow](#release-workflow)
  - [Release Ordering](#release-ordering)
  - [Tag Naming](#tag-naming)

<!-- /MarkdownTOC -->

## Versioning

Releases of these libraries are versioned using
[semantic versioning](https://semver.org). Where `a.b.c-tag` is the version
string, `a` is the major version, `b`, is the minor version, `c` is the patch
version, and `tag` is additional metadata, the following hold:

- Breaking changes to the public API will result in a major version increase.
- Addition of functionality in a backwards-compatible manner will result in a
  minor version increase.
- Backwards-compatible bug fixes will result in a patch version increase.
- The tag will indicate pre-release or beta versions, and will increase when any
  pre-release change is made. These are not intended to be stable.

## Release Branches

Releases are made from the `main` branch in this repository, and each release to
[`crates.io`](https://crates.io) must correspond to a tag on the `main` branch.

## Release Workflow

Cutting a release for these libraries works as follows:

1.  PR your changes against `main`, ensuring that the modified libraries have a
    bumped version number.
2.  Once the PR lands on `main`, create a tag for each library you are
    releasing, named as described below in [tag naming](#tag-naming).
3.  Release each of these libraries to [`crates.io`](https://crates.io). If you
    are a member of the Enso team and do not have the required permissions,
    please ask.

If a change to a library (e.g. prelude) is not required in the downstream
libraries in this repo and additionally _does not cause breakage_ of said
downstream libraries, it is not necessary to release new versions of the
downstream libraries too.

### Release Ordering

As the crates in this repository are interdependent, you will need to release
them in the following order as `cargo` does not yet have support for
interdependent releases, and the `cargo publish-all` project doesn't seem to
work.

1.  `enso-macro-utils`
2.  `enso-generics`
3.  `enso-shapely-macros`
4.  `enso-shapely`
5.  `enso-prelude`
6.  `enso-data`
7.  `enso-optics`
8.  `enso-logger`
9.  `enso-automata`

> The actionables for this section are:
>
> - Record the release order here.

### Tag Naming

Releases are made for the individual libraries in this repository and are made
available on [crates.io](https://crates.io). Each release must correspond to a
_tag_ in the repository using the following format: `libname-version` where:

- `libname` is the name of the library (as it is imported).
- `version` is the version string using the [semver](https://semver.org/)
  version scheme.
