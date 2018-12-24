class @I32(i: LlvmNumber)
  def add(other: @I32): @I32
  end

  def to_num(): Number
  end
end


class Number(num: @I32)
end


def main(): Number
  val a = @I32(118)
  @I32(12).add(a).to_num()
end
