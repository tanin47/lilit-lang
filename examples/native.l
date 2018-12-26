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
  def is_greater_than(other: Number): Boolean
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

def main(): Number
  val age = @I32:strtol(@String:read(), @I32(0), @I32(10)).to_num()
  if age.is_greater_than(@I32(10).to_num()) {
    @Void:printf+(@String("That's old"))
  } else {
    @Void:printf+(@String("That's not old"))
  }
  1
end
