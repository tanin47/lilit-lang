class @I8(i: Number)
  def add(other: @I8): @I8
  end

  def to_num(): Number
  end
end

def main(): Number
  val first = @I8(13)
  val result = @I8(1).add(first)
  result.to_num()
end
