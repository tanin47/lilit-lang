Ode
=======

Ode is a statically-typed and beautifully-terse programming language. It can compile to a single executable that runs on a target CPU. Thus, Ode is great for building command-line tools.


Principles
-----------

### Statically typed

We believe that a statically typed language, as codebase grows bigger, is exponentially more maintainable than a dynamically-typed language. I've experienced this pain first hand when working on a large Python codebase at Google.

### Beautifully terse

We aim to reduce the amount of code programmers need to write. For example, type inference is essential to avoid the verbosity problem in Java (e.g. `Animal animal = new Animal()`).

We also aim to provide a rich standard library to prevent programmers from solving trivial problems on their own. For example, in Python, programmers implement their own [getting the first element or null](https://stackoverflow.com/questions/363944/python-idiom-to-return-first-item-or-none).

### Maintainability over speed

We value maintainability over speed. For example, we might not implement the asynchronous programming paradigm because coding explicit yield point (e.g. with Monads) makes codebase less comprehensible. Another example is that we will not allow programmers to maintain their own memory to avoid various problems that come with it (e.g. memory corruption).


Features
---------

### Compile to a target CPU

Ode code can be compiled to a single executable that runs on a target CPU. This is ideal for deploying a command-line tool on user's machine.

### Tree shaking

Because the standard libary is big and will continue to grow in size, we perform tree shaking to remove the parts that are not used and, thus, reduce the size of an executable.


Write your first Ode
---------------------

```
print "Sawasdee!"
```
