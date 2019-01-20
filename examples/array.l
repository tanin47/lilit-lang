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

class @String(s: LlvmString)
end

class String(s: @String)
end

class @Void()
end


class @Array(array: LlvmArray)
  def get(index: @I32): String
  end
end

class Array(array: @Array, size: Number)
  def get(index: Number): String
    array.get(index.num)
  end
end

def main(args: Array): Number
  @Void:printf+(args.get(0).s)
  @Void:printf+("\n".s)
  @Void:printf+(args.get(1).s)
  @Void:printf+("\n".s)
  @Void:printf+(args.get(2).s)
  @Void:printf+("\n".s)
  @Void:printf+(args.get(3).s)
  @Void:printf+("\n".s)
  val a = ["first", "second"]
  @Void:printf+(a.get(1).s)
  @Void:printf+("\n".s)
  1
end
