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

We will be using it on a few codebases including [Rust](github.com/rust-lang/rust), [Servo](https://github.com/servo/servo), [Gathering Our Voices](https://github.com/BCAAFC/Gathering-Our-Voices), [socket.io](http://socket.io/), [connect](https://github.com/senchalabs/connect).

### Milestones

Date | Milestone | Complete
----------- | ------------- | -----
February 3 | Initial prototype of project system | Yes
February 10 | Well-defined project output | Yes
February 12 | Feature freeze | Yes
February 16 | Complete refactor identification functionality | In progress
February 17 | Complete testing & release version 1.0 |
February 20 | Complete analysis of target codebases |
February 21 | Document findings |
February 22 | Finalized report |
February 23 | Submit final project |

### Resources

* [git2 library](http://alexcrichton.com/git2-rs/git2/index.html)
* [Rust Book](http://doc.rust-lang.org/book/)
* [Rust Docs](http://doc.rust-lang.org/std/index.html)
* [Using Git Diff to detect Code Merges](http://stackoverflow.com/a/12805390)
* [Linus on tracking moves in Git](http://article.gmane.org/gmane.comp.version-control.git/217)
* [Content Tracking](https://gitster.livejournal.com/35628.html)
* [Git Revwalk](http://ben.straub.cc/2013/10/02/revwalk/)
