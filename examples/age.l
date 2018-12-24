class @String(s: String)
end

class @Boolean(b: Boolean)
  def to_bool(): Boolean
  end
end

class @I32(num: Number)
  def is_greater_than(other: @I32): @Boolean
  end

  def to_num(): Number
  end
end

class Number(num: @I32)
  def is_greater_than(other: Number): Boolean
    num.is_greater_than(other.num).to_bool()
  end
end

class Boolean(bool: @Boolean)
end

def main(): Number
  @Void:printf+(@String("Please enter your age: "))
  val age = @I32:strtol(@String(@String:read()), @I8(0), @I32(10)).to_num()
  val msg = if age.is_greater_than(10) {
    "That's old"
  } else {
    "That's not too old"
  }
  @Void:printf+(@String(msg))
  val result = if age.is_greather_than(10) {
    99
  } else {
    10
  }
  result
end
