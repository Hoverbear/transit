# transit

[Class Project]

### Project Question

> As developers, we commonly restructure our code. This is usually done in one commit (otherwise it's sloppy). Can we track when different parts of codebases undergo "movement" during refactoring?

### Methodology

Given a functioning Git repo this tool will attempt to do the following things:

1. Analyze each diff.
2. Attempt to match any deletions with any additions which match the same *signature* of code. This would correspond to a 'code move'.
3. Ideally, this program would be able to account for relevant variable name changes without failing to detect the move.

> Accounting for changes in variable names is not yet implemented.

Since we built a tool, we did not perform significant gathering of outside metrics. Instead, we generated several test repositories which we used to verify the functionality of our tool.

#### Installing `transit`

> For Linux or Mac, with root! Windows users are, unfortunately, on their own.

You will need the [Rust](http://rust-lang.org/) compiler:

```bash
curl -L https://static.rust-lang.org/rustup.sh | sudo sh
```

This should install `cargo` and `rustc`. Clone the repository and build it:

```bash
git clone git@github.com:Hoverbear/transit.git && \
cd transit && \
cargo build
```

Now you can run the binary on any repository, even itself! It will output JSON.

```bash
./target/debug/transit .
```

#### Installing `transit-display`

To use the (optional) output viewer, install `nodejs` and `npm` using your preferred package manager. `brew`, `pacman`, `apt`, and `yum` all have official packages and they will do. Then:

```bash
cd transit-display &&
npm install &&
npm start
```

From there you can:

* Visit `http://localhost:3000/transit` in your web browser to see some mock data.
* Query a repo use an invocation such as `http://localhost:3000/transit?repopath=../test/basic&repo=transit`. Where `repopath` is the path to your repository, and `repo` is the title of the project (for display only).
* Visit `http://localhost:3000/select` to choose a repository.

### Results

We ran `transit` against the following repositories:

* [capnproto-rust](https://github.com/dwrensha/capnproto-rust)
* [rust-url](https://github.com/servo/rust-url)
* [git2-rs](https://github.com/alexcrichton/git2-rs/)
* [connect](https://github.com/senchalabs/connect)

The outputs are stored in `./examples_runs/`.

### Analysis

#### capnproto-rust

Due to the length of the output, the results from `capnproto-rust` is stored in a [json](https://github.com/Hoverbear/transit/blob/f4c33e652510310607032da7b28af0741e739b7f/example_runs/capn-proto.json) file.

Transit found 52 moves in this repository. Of those 52 moves, 30 were single line moves.

#### rust-url

![Image of output from transit ran against rust-url](https://github.com/Hoverbear/transit/blob/f4c33e652510310607032da7b28af0741e739b7f/example_runs/rust-url.png)

Transit found 8 moves in this repository. Of those 8 moves, 1 was a single line move. The majority of these moves were 100+ lines of code.

On closer inspection, the 3 line move in commit https://github.com/servo/rust-url@a1fdd28ec7761777c6d075bfe9974150a24c4d34 is actually a change in logic.

#### git2-rs

![Image of output from transit ran against git2-rs](https://github.com/Hoverbear/transit/blob/f4c33e652510310607032da7b28af0741e739b7f/example_runs/git2-rs.png)

Transit found 7 moves in this repository. Of those 7 moves, two were single line moves.

#### connect

![Image of output from transit ran against connect](https://github.com/Hoverbear/transit/blob/f4c33e652510310607032da7b28af0741e739b7f/example_runs/connect.png)

Transit found 91 moves in this repository. Of those 91 moves, 42 were single line moves.

#### Overall

Transit is successful in detecting code moves.

Some of the detected moves where not simple refactoring but changes that would have changed the logic of the analyzed programs. It is worth noting that beyond our small test data, we did not check the percentage of moves that were not detected by `transit`.

### Project Management

Team Member | Github Account
----------- | --------------
Andrew Hobden | @Hoverbear
Brody Holden | @BrodyHolden
Fraser DeLisle | @fraserd

Date | Milestone | Complete
----------- | ------------- | -----
February 3 | Initial prototype of project system | Yes
February 10 | Well-defined project output | Yes
February 12 | Feature freeze | Yes
February 17 | Complete refactor identification functionality | Yes
February 19 | Complete testing & release version 1.0 | Yes
February 21 | Complete analysis of target codebases | Yes
February 21 | Document findings | Yes
February 22 | Finalized report | Yes
February 23 | Submit final project | Yes

A break down of which work tasks were completed by which team members is tracked in issue [#2](https://github.com/Hoverbear/transit/issues/2).

### Resources

* [git2 library](http://alexcrichton.com/git2-rs/git2/index.html)
* [Rust Book](http://doc.rust-lang.org/book/)
* [Rust Docs](http://doc.rust-lang.org/std/index.html)
* [Using Git Diff to detect Code Merges](http://stackoverflow.com/a/12805390)
* [Linus on tracking moves in Git](http://article.gmane.org/gmane.comp.version-control.git/217)
* [Content Tracking](https://gitster.livejournal.com/35628.html)
* [Git Revwalk](http://ben.straub.cc/2013/10/02/revwalk/)
