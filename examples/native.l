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
end


def main(): Number
  if @I32(12).to_num().is_greater_than(@I32(11).to_num()) {
    10
  } else {
    20
  }
end
