# transit

[Class Project]

[Incomplete]

### Project Question

> As developers, we commonly restructure our code. This is usually done in one commit (otherwise it's sloppy). How often do different parts of codebases undergo "movement" during refactoring? What is the overall complexity cost of maintaining a familiarity with a given system undergoing these changes?

### Methodology

Given a functioning Git repo this tool will attempt to do the following things:

1. Analyze each diff.
2. Attempt to match any deletions with any additions which match the same *signature* of code. This would correspond to a 'code move'.
3. Ideally, this program would be able to account for relevant variable name changes without failing to detect the move.

### Codebases to Analyze

I'll be using it on a few codebases including [Rust](github.com/rust-lang/rust), [Servo](https://github.com/servo/servo), and my own project [Gathering Our Voices](https://github.com/BCAAFC/Gathering-Our-Voices).

### Milestones

Febuary 23 - Hand in.

### Resources

* [git2 library](http://alexcrichton.com/git2-rs/git2/index.html)
* [Rust Book](http://doc.rust-lang.org/book/)
* [Rust Docs](http://doc.rust-lang.org/std/index.html)
