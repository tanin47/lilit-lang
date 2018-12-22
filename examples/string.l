class @I8(c: Number)
  // No implementation. This is bound to the code in the compiler.
  abstract def ==(other: @I8)
  abstract def +(other: @I8)
end

// We cannot have these @I8 or other LLVM classes become Lilit code. It has to be implemented in the compiler.
// But, then, `chars[0] == @I8(c)` won't work. How do we resolve this?
//
// LLVM class needs to provide that method.
//
// String is too complex. Let's implement Number first.

class String(chars: @Array[@I8])
  def startsWith(c: Char)
    // chars[0] needs to have LLVM support
    // How can we also compare between these 2 LLVM types?
    // LLVM types are special.
    // We need a refactoring to support these LLVM types properly.
    chars[0] == @I8(c)
  end
end

def main(): Number
  val a = String("Allie", 10)
  @Void:printf+(@String(a.name))
  a.run()
end
