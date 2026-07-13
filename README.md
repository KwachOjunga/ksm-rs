# Kisumu Lang (Rust Implementation but with a twist).

The following is a very quick and dirty implementation fo kisumu lang.
The code base exists in the following parts;
  - a repl binary module that does whatever its name implies
  - a compiler that is capable of emitting llvm given a program utilising a set
   of the currently defined language features.

  It is safe to say that most of what the language is capable of will be defined 
  by a set of [examples](./src/bin/klc/examples/) in defined in klc bin module.

  To try observe the current llvm output one can run:

  ensure you have atleast the stable version of rust installed.

```sh
    cargo r --bin klc --input src/bin/klc/examples/fibonacci.kl --arg 7 
```

Or

```sh
    cargo r --bin klc --input src/bin/klc/examples/factorial.kl --arg 5 
```

::NOTE:: As far as the quality of the llvm output currently stands, it is suboptimal.
