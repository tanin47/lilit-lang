; ModuleID = 'main'
source_filename = "main"

%"@I32" = type { i32 }
%Number = type { %"@I32"* }

@formatString = private constant [13 x i8] c"hella %d %p\0A\00"
declare i32 @printf(i8*, ...)

declare noalias i8* @malloc(i64) #1

define %Number* @"__@I32__to_num"(%"@I32"*) {
first_block:
  %number_malloc = call noalias i8* @malloc(i64 8) #2
  %number = bitcast i8* %number_malloc to %Number*

  %first_param_ptr = getelementptr inbounds %Number, %Number* %number, i32 0, i32 0
  store %"@I32"* %0, %"@I32"** %first_param_ptr

  ret %Number* %number
}

define %"@I32"* @__Number__get(%Number*, %Number*) {
first_block:
  %param = alloca %Number*
  %param1 = alloca %Number*

  store %Number* %0, %Number** %param
  ; store %Number* %1, %Number** %param1

  %zero_gep = getelementptr %Number, %Number* %0, i32 0, i32 0
  %zero_address = bitcast %Number* %0 to i8*
  %call1 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @formatString , i32 0, i32 0), i32 111, i8* %zero_address)

  %i32_zero_ptr = load %"@I32"*, %"@I32"** %zero_gep
  %i32_zero_ptr_address = bitcast %"@I32"* %i32_zero_ptr to i8*
  %call8 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @formatString , i32 0, i32 0), i32 222, i8* %i32_zero_ptr_address)

  %i32_zero_first_ptr = getelementptr %"@I32", %"@I32"* %i32_zero_ptr, i32 0, i32 0
  %d_zero = load i32, i32* %i32_zero_first_ptr
  %d_zero_address = bitcast i32* %i32_zero_first_ptr to i8*
  %call3 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @formatString , i32 0, i32 0), i32 %d_zero, i8* %d_zero_address)

  %as = alloca i32
  ret %"@I32"* %i32_zero_ptr
}

define i32 @main() {
first_block:
  %class = alloca %"@I32"
  %"gep for member" = getelementptr inbounds %"@I32", %"@I32"* %class, i32 0, i32 0
  store i32 18, i32* %"gep for member"
  %to_num = call %Number* @"__@I32__to_num"(%"@I32"* %class)

  %class2 = alloca %"@I32"
  %some2 = getelementptr inbounds %"@I32", %"@I32"* %class2, i32 0, i32 0
  store i32 18, i32* %some2
  %to_num2 = call %Number* @"__@I32__to_num"(%"@I32"* %class2)

  %ret = call %"@I32"* @__Number__get(%Number* %to_num, %Number* %to_num2)

  %i32_ret_gep = getelementptr inbounds %"@I32", %"@I32"* %ret, i32 0, i32 0
  %ret_i32 = load i32, i32* %i32_ret_gep
  %ret_i32_address = bitcast i32* %i32_ret_gep to i8*
  %call2 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @formatString , i32 0, i32 0), i32 %ret_i32, i8* %ret_i32_address)

  ret i32 %ret_i32
}
