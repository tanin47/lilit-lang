def main()
  print("Please enter your age: ")
  val age = parse(read(1))
  if age > 35 {
    print("That's old")
  } else {
    print("That's not too old")
  }
  val result = if age > 10 {
    99
  } else {
    10
  }
  result
end
