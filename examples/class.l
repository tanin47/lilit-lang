class @I32(i: LlvmNumber)
end

class Number(num: @I32)
end

class Void()
end

class Animal(name: String)
  def run(): Number
    @Void:printf+(@String(name))
    14
  end
end

def main(): Number
  val a = Animal("Allie")
  @Void:printf+(@String(a.name))
  1
end
