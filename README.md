Haiku
=======

Haiku is a statically-typed and beautifully-terse programming language. It can compile to a single executable that runs on a target CPU. Thus, Haiku is great for building command-line tools.


Principles
-----------

### Statically typed

We believe that a statically typed language, as codebase grows bigger, is exponentially more maintainable than a dynamically-typed language. I've experienced this pain first hand when working on a large Python codebase at Google.

### Beautifully terse

We aim to reduce the amount of code programmers need to write. For example, type inference is essential to avoid the verbosity problem in Java (e.g. `Animal animal = new Animal();` can be reduced to `animal = new Animal()`).

We also aim to provide a rich standard library to prevent programmers from solving trivial problems on their own. For example, in Python, programmers have to implement their own [getting the first element or null](https://stackoverflow.com/questions/363944/python-idiom-to-return-first-item-or-none), while, in Ruby, they can use `.first` in Ruby's standard library.

### Maintainability over speed

We value maintainability over speed. For example, we might not implement the asynchronous programming paradigm because coding explicit yield point (e.g. with Monads) makes codebase less comprehensible. Another example is that we will not allow programmers to manage their own memory to avoid various problems that come with it (e.g. memory corruption).


Features
---------

* Compile to a target CPU (ideal for deploying a command-line tool)
* Tree shaking (reducing the size of the binary)
* No null; only optional type
* Complex type system (e.g. strongly generic, multiple inheritance)
* Metaprogramming


Write your first Haiku
---------------------

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
