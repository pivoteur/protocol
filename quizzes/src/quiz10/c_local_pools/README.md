# `c_local_pools`

How do I run `pools` locally?

## Context

`pools` used to query git to get the active pools on the protocol. I find this
gives `pools` too much access to the repository, no matter how careful one is.

## Locally

Therefore, revise `pools` to read the active pools from the local 
directory-system, and have git manage what is visibly local to `pools`.

Security-issue solved.

## New `pools`

`$ pools <dir>`

* where `<dir>` is the directory where the active pools reside.

