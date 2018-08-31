Language features
---------------------

* Everything is an expression
* First-class method
* Support traits
* Support polymorphism
* Overload method


Examples of class and inheritance
----------------------------------

```
class Animal extends CanWalk, CanSpeak
  def init(a: Int, b: String)

  end


end
```

```
trait CanWalk
  def walk()

  end
end

trait CanSkipWalk
  def walk()

  end

  def walk(speed: Int)

  end
end

trait CanSpeak
  def speak()

  end
end
```


