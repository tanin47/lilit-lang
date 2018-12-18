def main(): Number
  @Void:printf+(@String("Please enter your age: "))
  val age = @I32:strtol(@String(@String:read()), @Pointer(@Pointer(@I8(0))), @I32(10))
  val msg = if age > 35 {
    "That's old"
  } else {
    "That's not too old"
  }
  @Void:printf+(@String(msg))
  val result = if age > 10 {
    99
  } else {
    10
  }
  result
end
