class @Boolean(b: LlvmBoolean)
  def to_boolean(): Boolean
  end
end

class @I32(i: LlvmNumber)
  def add(other: @I32): @I32
  end

  def is_greater_than(other: @I32): @Boolean
  end

  def to_num(): Number
  end
end

class Boolean(b: @Boolean)
end

class Number(num: @I32)
  def >(other: Number): Boolean
    num.is_greater_than(other.num).to_boolean()
  end

  def add(other: Number): Number
    num.add(other.num).to_num()
  end
end

class @Char(c: LlvmChar)
end

class Char(c: @Char)
end

class @String(s: LlvmString)
  static def from(s: String): @String

  end
end

class String(chars: Array)
end

class @Void()
end

class @Array(array: LlvmArray)
  def get(index: @I32): Char
  end
end

class Array(array: @Array, size: Number)
  def get(index: Number): Char
    array.get(index.num)
  end
end

def main(args: Array): Number
  val s = "test string"
  @Void:printf+(@String.from(s))
  @Void:printf+(@String.from("\n"))
  1
end
