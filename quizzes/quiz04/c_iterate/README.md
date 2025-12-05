# c_iterate

Now that we have the pivot pool assets, let's iterate over each pool and run a 
report of all pivot pools in one go!

[tweet](https://x.com/pivocateur/status/1996725018080772504)

Let's break down this breakdown.

* First, let's just [replicate fetching pool names from 
git](src/main.rs) after moving that functionality to the 
[libraries](https://github.com/pivoteur/protocol/blob/iterate-pools/libs/src/git.rs#L12)

![Fetching pivot pool names from git](imgs/01-git-pools.png)

* Next, well: let's iterate and recommend close pivots from each pool!

