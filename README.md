Lilit
=======

Lilit is a statically typed and beautifully terse programming language that compiles to a single executable.

Lilit in Thai (ลิลิต) is [a Thai literary genre](http://cuir.car.chula.ac.th/handle/123456789/51485). 'Lilit' comes from 'Lalit' in Pali and Sansakrit languages. It means 'to play': to play rhythmic positions which have the same tone.


Principles
-----------

### Statically typed

A statically typed language, as codebase grows bigger, is exponentially more maintainable than a dynamically-typed language. I've experienced this pain first hand when working on a large Python codebase at Google.

### Beautifully terse

We aim be at the highest level of abstraction and, thus, reduce the amount of detail programmers need to think about. 

The first example is type inference, which is essential to avoid the verbosity problem in Java (e.g. `Animal animal = new Animal();` can be reduced to `animal = Animal()`).

The second example is complex type system. A complex type system (think Scala), though taking effort to learn, enables programmers to capture real-world complexity with brevity.

We also aim to provide a rich standard library to refrain programmers from solving trivial problems on their own. For example, in Python, programmers have to implement their own [getting the first element or null](https://stackoverflow.com/questions/363944/python-idiom-to-return-first-item-or-none), while, in Ruby, they can use `.first` in Ruby's standard library.

While Lilit is a general-purpose programming language, I think Lilit is gonna be great for building command-line tools.


Features
---------

* Compile to a target CPU (ideal for deploying a command-line tool)
* Tree shaking (removing unused methods and, thus, reducing the size of the binary)
* No null; only optional type
* Complex type system (e.g. strongly generic, multiple inheritance)
* Metaprogramming


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


1. Compile using `cargo run examples/native.l`.
2. Compile native code: `llc-6.0 -filetype=obj native/lib.ll`.
2. Link it using ` cc native/lib.o output/main.o ~/projects/bdwgc/.libs/libgc.so -I ~/projects/bdwgc/include/ -no-pie -o main`.
3. Run `./main`.
