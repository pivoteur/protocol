# quiz09

## Integration testing

Now, as a quiz, I want to do integration-testing, but since I have functional 
tests in place as a separate entity from #[cfg(test)], I wish to do the same 
thing with integration tests, something like:

```PYTHON
for dapp in dir:
       results.add(confirm(cargo build))

report(results)
```

### Future work?

At some point do we want to simulate command line arguments or actually
call each dapp with appropriate command line arguments? ... that is to say:
more than just build each dapp, run each dapp?

-----

* [a_itr](a_itr): iterate through subdirectories and execute `cargo build`
and see what kind of results we get

