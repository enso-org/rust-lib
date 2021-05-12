---
layout: developer-doc
title: Contributing to the Enso Rust Libraries
category: summary
tags: [summary, contributing]
order: 1
---

# Contributing to This Repository

Thank you for your interest in contributing to these libraries! We believe that
only through community involvement can we make them the best they can be! There
are a whole host of ways to contribute, and every single one is appreciated. The
major sections of this document are linked below:

<!-- MarkdownTOC levels="2" autolink="true" -->

- [The Contributor License Agreement](#the-contributor-license-agreement)
- [Issues](#issues)
- [Bug Reports](#bug-reports)
- [Hacking on These Libraries](#hacking-on-these-libraries)
- [Developing in Conjunction with Enso / IDE](#developing-in-conjunction-with-enso--ide)
- [Pull Requests](#pull-requests)
- [Issue Triage](#issue-triage)

<!-- /MarkdownTOC -->

All contributions to Enso should be in keeping with our
[Code of Conduct](https://github.com/enso-org/enso/docs/CODE_OF_CONDUCT.md).

## The Contributor License Agreement

As part of your first contribution to this repository, you need to accept the
Contributor License Agreement. You will automatically be asked to sign the CLA
when you make your first pull request.

Any work intentionally submitted for inclusion in Enso shall be licensed under
this CLA.

The CLA you sign applies to all repositories associated with the Enso project,
so you will only have to sign it once at the start of your contributions.

## Issues

If you're wanting to get involved with Enso's development and are looking for
somewhere to start, you can check out the following tags in our issues:

- [Good First Issue](https://github.com/enso-org/rust-lib/labels/Status%3A%20Good%20First%20Issue)
- [Help Wanted](https://github.com/enso-org/rust-lib/labels/Status%3A%20Help%20Wanted)

You can use the "Size" and "Difficulty" labels that should be assigned to every
issue to get a better idea of how much work a given issue might be.

## Bug Reports

While it's never great to find a bug, they are a reality of software and
software development! We can't fix or improve on the things that we don't know
about, so report as many bugs as you can! If you're not sure whether something
is a bug, file it anyway!

**If you are concerned that your bug publicly presents a security risk to the
users of Enso, please look at our [security guidelines](./SECURITY.md).**

Even though GitHub search can be a bit hard to use sometimes, we'd appreciate if
you could
[search](https://github.com/enso-org/rust-lib/search?q=&type=Issues&utf8=%E2%9C%93)
for your issue before filing a bug as it's possible that someone else has
already reported the issue. We know the search isn't the best, and it can be
hard to know what to search for, so we really don't mind if you do submit a
duplicate!

Opening an issue is as easy as following
[this link](https://github.com/enso-org/rust-lib/issues/new?template=bug-report.md)
and filling out the fields. The template is intended to collect all the
information we need to best diagnose the issue, so please take the time to fill
it out accurately.

The reproduction steps and the version information are particularly important,
as the more easily we can reproduce it, the faster we can fix the bug!

## Hacking on These Libraries

This will get you up and running for working on these libraries, with only a
minimal amount of setup required.

### System Requirements

In order to build these libraries you will need the following tools.

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html),
  the standard rust build tool.
- [Rustup](https://rustup.rs) for managing rust toolchains.
- [Node](https://nodejs.org/en/) if you want to test the libraries with WASM.
- [wasm-pack](https://github.com/rustwasm/wasm-pack) for building WASM versions
  of the libraries and running WASM tests.

We only support the current LTS node version. This is currently `v12.18.4`.

### Getting the Sources

Given you've probably been reading this document on GitHub, you might have an
inkling where to look!. You can clone Enso using two methods:

- **Via HTTPS:** We recommend you only use HTTPS if checking out the sources as
  read-only.

```
git clone https://github.com/enso-org/rust-lib.git
```

- **Via SSH:** For those who plan on regularly making direct commits, cloning
  over SSH may provide a better user experience (but requires setting up your
  SSH Keys with GitHub).

```
git clone git@github.com:enso-org/rust-lib.git
```

### Getting Set Up (Rust)

You can get set up by running the following commands in the cloned repository.

```bash
rustup toolchain install nightly-2021-05-12
rustup override set nightly-2021-05-12
rustup component add clippy
```

### Building the Libraries

Building these libraries is as simple as using the cargo build tool. You can run
`cargo build` to build all of the libraries, or `cargo build $lib`, where `$lib`
is the name of one of the libraries in this project.

#### Troubleshooting

If you are having issues building the libraries, please check the list below
before filing an issue with us.

- `error[E0554]`: `#![feature]` may not be used on the \$chan release
  channel`: The version of `rustc`seen by`cargo` is not a nightly build, and a
  nightly build is required.
- **Compile Errors:** Please check that you are on the correct `rustc` nightly
  (`nightly-2021-05-12`).

If your problem was not listed above, please
[file a bug report](https://github.com/enso-org/rust-lib/issues/new?assignees=&labels=Type%3A+Bug&template=bug-report.md&title=)
in our issue tracker and we will get back to you as soon as possible.

## Developing in Conjunction with Enso / IDE

As these libraries are used frequently by both the
[Enso language](https://github.com/enso-org/enso) and the
[Enso IDE](https://github.com/enso-org/ide), it is often necessary to develop
these libraries alongside those projects. This section deals with the workflow
for doing so.

### Patch Overrides

[Cargo](https://doc.rust-lang.org/cargo/), the Rust package manager and build
tool, has support for
[overriding dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html).
The best way that we've found to use this functionality for developing these
libraries in conjunction with another project is to use a
[patch version override](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section).

As the patch paths care able to point _outside_ the current Cargo project, you
can check out this repository and the other one side by side and set the patch
paths for the libraries you want to develop.

The following is an example:

```toml
[patch.crates-io]
enso-automata       = { path = "../rust-lib/src/automata"       }
enso-data           = { path = "../rust-lib/src/data"           }
enso-generics       = { path = "../rust-lib/src/generics"       }
enso-logger         = { path = "../rust-lib/src/logger"         }
enso-macro-utils    = { path = "../rust-lib/src/macro-utils"    }
enso-optics         = { path = "../rust-lib/src/optics"         }
enso-prelude        = { path = "../rust-lib/src/prelude"        }
enso-shapely        = { path = "../rust-lib/src/shapely/impl"   }
enso-shapely-macros = { path = "../rust-lib/src/shapely/macros" }
```

### PR Workflow

This does, however, create a bit of a dependency for the order in which you
submit your PRs. We recommend working as follows if you need to develop one of
these libraries alongside a project.

1.  Clone this repository alongside the project repository, such that it is at
    the relative path `../rust-lib`.
2.  In your project's `Cargo.toml`, add a patch override for each library in
    this repo that you want to work on.
3.  When you are done, you need to bump the library versions for the changed
    libraries, and PR your changes to this repo. Make sure that you _only_ do
    this after you're done, as before can cause some issues with `Cargo.lock`.
4.  Once the PR is accepted, you need to follow the
    [release process](./release-policy.md) to release new versions to
    [`crates.io`](https://crates.io).
5.  Once those versions are released, you need to first remove the patch
    overrides and then update the versions in your project.
6.  You can now PR your changes to the main project.

## Pull Requests

Pull Requests are the primary method for making changes to these libraries.
GitHub has
[fantastic documentation](https://help.github.com/articles/about-pull-requests/)
on using the pull request feature. This repo uses the 'fork-and-pull' model of
development. It is as described
[here](https://help.github.com/articles/about-collaborative-development-models/)
and involves people pushing changes to their own fork and creating pull requests
to bring those changes into the main Enso repository.

Please make all pull requests against the `main` branch.

- We run CI on all contributions to this repo, but it's still useful for you to
  run the tests yourself locally first! This can be done by running `cargo test`
  or by running `cargo test $lib`, where `$lib` is the name of the library.
- Additionally, please ensure that your code conforms to the Enso style guide
  for [Rust](https://github.com/enso-org/enso/docs/style-guide/rust.md).

Make sure you perform these checks before _every_ pull request. You can even add
[git hooks](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks) before
every push to make sure that you can't forget.

- Every pull request to the Enso repository is reviewed by a member of the core
  team! You'll get assigned a reviewer based on the areas your PR touches, but
  please feel free to ask for a specific person if you've worked with them in a
  specific area before!
- If you have questions, or would like to begin the review process before your
  PR is 'done', please use the
  [Draft Pull Requests](https://github.blog/2019-02-14-introducing-draft-pull-requests/)
  feature on GitHub. Doing so will allow you to make use of our CI
  infrastructure as part of your development process.

Once the reviewer approves your pull request it will be tested by our continuous
integration provider before being merged. If we request changes to your PR,
please feel free to discuss the suggestions and comments! We can only achieve
the best results through open collaboration.

## Issue Triage

Sometimes issues can be left open long after the bug has been fixed. Other
times, a bug might go stale because something has changed in the meantime.

It can be helpful to go through older bug reports and make sure that they are
still valid. Load up an older issue, double check that it's still true, and
leave a comment letting us know if it is or is not. The
[least recently updated](https://github.com/enso-org/rust-lib/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-asc)
sort is good for finding issues like this.

Contributors with sufficient permissions can help by adding labels to help with
issue triage.

If you're looking for somewhere to start, take a look at the
[Difficulty: Beginner](https://github.com/enso-org/rust-lib/labels/Difficulty%3A%20Beginner)
issue label, as well as the
[Status: Help Wanted](https://github.com/enso-org/rust-lib/labels/Status%3A%20Help%20Wanted)
and
[Status: Good First Issue](https://github.com/enso-org/rust-lib/labels/Status%3A%20Good%20First%20Issue)
labels.
