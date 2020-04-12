Lilit
=======

Lilit is a typed and terse programming language that compiles to a single executable.

While Lilit is a general-purpose programming language, Lilit aims to be ideal for building low-performant command-line tools.


Principles
-----------

### Typed

A statically typed language, as codebase grows bigger, is more maintainable than a dynamically-typed language.

### Terse

We aim be at the highest level of abstraction and, thus, reduce the amount of detail programmers need to code.

Some features that Lilit offers:

* Complex type system (think Scala), which enables programmers to capture real-world complexity with brevity, though it takes effort to learn.
* Rich standard library (think Scala + Ruby), which prevents programmers from solving trivial problems on their own. For example, in Python, programmers have to implement their own [getting the first element or null](https://stackoverflow.com/questions/363944/python-idiom-to-return-first-item-or-none), while, in Ruby, they can use `.first` in Ruby's standard library.


Features
---------

* Compile to a target CPU (ideal for deploying a command-line tool)
* No null; only optional type
* Complex type system (e.g. strongly generic, multiple inheritance)
* Tree shaking (removing unused methods and, thus, reducing the size of the binary)
* Limited metaprogramming


Write your first Lilit
------------------------

```
val name = "world"
print s"Hello $name"

if name == "world"
  print "This is not a person"
end

val car = Some("Subaru")

car.isDefined

class Test extends Base, Animal
  def init

  end
end
```


Build
------

1. Compile using `cargo run examples/native.lilit`.
2. Compile native code: `llc-6.0 -filetype=obj native/lib.ll`.
3. Link it using ` cc native/lib.o output/main.o ~/projects/bdwgc/.libs/libgc.so -I ~/projects/bdwgc/include/ -no-pie -o main`.
4. Run `./main`.

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
