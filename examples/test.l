def main()
  56
  another()
  12
end

class Test
	def test()
	  1
	end
end

def another()
  56
end

class Yo extends Yeah, Yay
	def test()
	  1
	end

	def test_another()
	  1
	  test()
	end
end

class EmptyClass
end
