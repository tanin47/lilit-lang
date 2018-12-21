class Animal(name: String, age: Number)
  def run(): Number
    @Void:printf+(@String(name))
    14
  end
end

def main(): Number
  val a = Animal("Allie", 10)
  @Void:printf+(@String(a.name))
  a.run()
end
