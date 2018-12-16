def main()
  @Void:printf+(@String("Type something: "))
  val input = @String:read()
  @Void:printf+(@String("You typed: "))
  @Void:printf+(@String(input))
  1
end

