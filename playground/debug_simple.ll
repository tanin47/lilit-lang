; ModuleID = 'main'
source_filename = "main"

define i32 @__Number__get(i32*, i32*) {
first_block:
  %param = alloca i32*
  store i32* %0, i32** %param
  %param1 = alloca i32*
  store i32* %1, i32** %param1
  %num_ptr = load i32*, i32** %param
  %num = load i32, i32* %num_ptr
  ret i32 %num
}

define i32 @main() {
first_block:
  %num = alloca i32
  store i32 29, i32* %num
  %get = call i32 @__Number__get(i32* %num, i32* %num)
  ret i32 %get
}
