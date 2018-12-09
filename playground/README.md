Playground
===========

The best way of learning LLVM is to write C code and compile it to LLVM IR.

We can learn how Clang constructs, for example, an array or a string in LLVM IR.

I think this is why LLVM IR's tutorial is almost non-existent. It's much faster to learn LLVM IR through Clang.


Usage
------

1. Modify `test.c`
2. Run Clang: `clang -S -emit-llvm test.c`
3. See LLVM IR in `test.ll`