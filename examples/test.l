def another(): Number
  10
end

def main(): Number
  @Void:printf+(@String("Type something: "))
  val input = @String:read()
  @Void:printf+(@String("You typed: "))
  @Void:printf+(@String(input))
  another()
end

