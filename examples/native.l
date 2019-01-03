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

def ask_for_age(question: String): Number
  @Void:printf+(question.s)
  @I32:strtol(@String:__lilit__read(), @I32(0), @I32(10)).to_num()
end

def main(): Number
  @Void:printf+("What's your name?: ".s)
  val name = @String:__lilit__read()
  @Void:printf+("Hello ".s)
  @Void:printf+(name)

  if ask_for_age("How old are you?: ") > 34 {
    @Void:printf+(@String("That's old\n"))
  } else {
    @Void:printf+(@String("That's not old\n"))
  }
  1
end
