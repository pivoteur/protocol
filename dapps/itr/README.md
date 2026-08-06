# itr

Integration test-suite that runs `cargo build` over each sub-directory in
`<dir>` and reports build success.

![Integrate tests](imgs/01-itr.png)

[src](../../quizzes/src/quiz09/a_itr/mod.rs)

-----

## Revisions

* 1.04, 2026-08-06: Do not print banner twice when `-h|--help` flagged
* 1.03, 2026-07-29: prints banner during integration-build
* 1.02, 2026-07-05: clap to process arguments and for usage-documentation
* 1.01, 2026-05-06: using new functional test framework
* 1.00, 2026-01-28: released!

