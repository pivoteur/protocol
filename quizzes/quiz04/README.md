# quiz 04

A step toward automating for all pivot pools is knowing  all the pivot pool names.

1. I COULD have a list of all pivot pools, but that's data telling me what data is: a redundancy.

2. Let's use the git API instead: https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28 

Find all pivot pool names with this.

## Pieces be with you

As per usual, I break the tasks into smaller tasks, then I break the smaller 
tasks into smaller-ER tasks when I find the smaller tasks to be way too huge to
solve on their own.

So it goes.

* [git sum!](a_git_json): First, let's just get the json from git, shall we?
* [parse JSON](b_parse): Next, let's parse the JSON names to Rust String-pairs.

