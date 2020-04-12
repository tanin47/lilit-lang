Lilit
=======

Lilit is a high-level general-purpose programming language.

```
def main(args: Array[String]): Void
  println("Hello Lilit!")
end
```

It aims to be ideal for building low-performant command-line tools.

Please follow our progress [here](./PROGRESS.md).

Principles
-----------

### Typed

A statically typed language, as codebase grows bigger, is more maintainable than a dynamically-typed language.

### Terse

We aim be at the highest level of abstraction and reduce the amount of detail programmers need to think and code.

Some features that Lilit offers:

* Complex type system (think Scala), which enables programmers to capture real-world complexity with brevity, though it takes effort to learn.
* Rich standard library (think Scala + Ruby), which prevents programmers from solving trivial problems on their own. For example, in Python, programmers have to implement their own [getting the first element or null](https://stackoverflow.com/questions/363944/python-idiom-to-return-first-item-or-none), while, in Ruby, they can use `.first` in Ruby's standard library.


Features
---------

* Compile to a single executable binary
* Automatic garbage collection
* Complex type system (e.g. generic, multiple inheritance, no null)
* Limited metaprogramming (e.g. type-safe monkey patching)


Run
------

Try `./run.sh`

Example:

```
$ cargo run examples/printf.lilit
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/lilit examples/printf.lilit`
Lilit 0.1.0

---- Code ----
class Native__Void
  // No implementation.
end

class Native__Int
  // No implementation. This class represents i64 in LLVM.
end

class Native__String
  // No implementation. This class represents i8* in LLVM.
end

def native__printf(text: Native__String): Native__Void
  // No implementation. The body is automatically built to invoke printf(..).
end

class Void
end

class Int(underlying: Native__Int)
end

class String(underlying: Native__String)
end

def println(text: String): Void
  native__printf(text.underlying)
end

def main: Int
  println("Hello world!")
  123
end

Write LLVM object to ./output/main.o

$ clang -S -emit-llvm /home/tanin/projects/bdwgc/.libs/libgc.so -I /home/tanin/projects/bdwgc/include/ -o native/lib.ll native/lib.c
clang: warning: /home/tanin/projects/bdwgc/.libs/libgc.so: 'linker' input unused [-Wunused-command-line-argument]

$ llc-6.0 -filetype=obj native/lib.ll

$ cc native/lib.o output/main.o /home/tanin/projects/bdwgc/.libs/libgc.so -I /home/tanin/projects/bdwgc/include/ -o main -no-pie

$ ./main
Hello world!

$ echo $?
123
```

Technical detail
-----------------

1. __Tokenize__ builds a sequence of predefined tokens from a sequence of characters. This stage simplifies Parsing, which is the next step.
2. __Parsing__ builds a parse tree from the sequence of tokens.
3. __Index__ builds an index tree, which enables references across all files. The index tree can answer a question like: "Can we invoke method A on class C?".
4. __Analyse__ populates references in the parse tree (e.g. populating a method call with the corresponding method definition).
5. __Emit__ builds LLVM code from the populated parse tree.

Development tricks
-------------------

### Use Clang to emit LLVM IR from C code

1. Write C code
2. Run `clang -S -emit-llvm test.c`
3. See how Clang build equivalent LLVM IR

### Debug segfault

When encountering the error like below:

```
$ cargo test emit::tests::test_full -- --nocapture
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running target/debug/deps/lilit-e4a1085d58b2f6af

running 1 test
error: process didn't exit successfully: `/home/tanin/projects/lilit-lang/target/debug/deps/lilit-e4a1085d58b2f6af 'emit::tests::test_full' --nocapture` (signal: 11, SIGSEGV: invalid memory reference)
```

We can use GDB to identify which line causes the memory corruption.

1. Run `gdb /home/tanin/projects/lilit-lang/target/debug/deps/lilit-e4a1085d58b2f6af 'emit::tests::test_full'`
2. Run `run` and see the memory corruption
3. Run `backtrace` to see which line causes the memory corruption.

We'd see a backtrace like below:

```
#0  0x000055555598a53c in LLVMBuildAlloca ()
#1  0x0000555555665a95 in inkwell::builder::Builder::build_alloca (self=0x7ffff6429520, ty=..., name=...)
    at /home/tanin/.cargo/git/checkouts/inkwell-9eb0689e3d3f00ac/46d576c/src/builder.rs:173
#2  0x000055555568a54c in <lilit::emit::Emitter as lilit::emit::def::method::EmitterMethod>::apply_method (self=0x7ffff6429518, method=0x7fffe0002aa8)
    at src/emit/def/method.rs:51
#3  0x000055555568ea53 in lilit::emit::Emitter::apply_file (self=0x7ffff6429518, file=0x7fffe0001a30) at src/emit/mod.rs:61
#4  0x000055555568e7fb in lilit::emit::Emitter::apply (self=0x7ffff6429518, files=...) at src/emit/mod.rs:46
...
```

FAQs
-----

### What does Lilit mean?

Lilit in Thai (ลิลิต) is [a Thai literary genre](http://cuir.car.chula.ac.th/handle/123456789/51485). 'Lilit' comes from 'Lalit' in Pali and Sansakrit languages. It means 'to play': to play rhythmic positions which have the same tone.
