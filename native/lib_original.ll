%struct._IO_FILE = type opaque
declare i8* @fgets(i8*, i32, %struct._IO_FILE*)

@stdin = external global %struct._IO_FILE*, align 8

declare %struct._IO_FILE* @fopen(i8*, i8*)

declare i64 @fread(i8*, i64, i64, %struct._IO_FILE*)

declare noalias i8* @GC_malloc(i64)

@read_file_mode = private unnamed_addr constant [2 x i8] c"r\0"

define i8* @__lilit__read() {
  %string =  call i8* @GC_malloc(i64 mul nuw (i64 ptrtoint (i8* getelementptr (i8, i8* null, i32 1) to i64), i64 100))
  %casted = bitcast i8* %string to [100 x i8]*
  %pointer = getelementptr inbounds [100 x i8], [100 x i8]* %casted, i32 0, i32 0
  %stdin = load %struct._IO_FILE*, %struct._IO_FILE** @stdin
  %dontcare = call i8* @fgets(i8* %pointer, i32 100, %struct._IO_FILE* %stdin)
  ret i8* %pointer
}

define void @__lilit__read_file(i8*, i8**) {
  %buf = alloca [1024 x i8]
  %file = call %struct._IO_FILE* @fopen(i8* %0, [2 x i8]* @read_file_mode)
  ret i8* %pointer
}

@finalizer.str.freed = private unnamed_addr constant [13 x i8] c"GC freed %d\0A\00"
@finalizer.count = internal global i32 0, align 4

define void @finalizer(i8*, i8*) {
  %count = load i32, i32* @finalizer.count, align 4
  %count2 = add nsw i32 %count, 1
  store i32 %count2, i32* @finalizer.count, align 4
  ; call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([13 x i8], [13 x i8]* @finalizer.str.freed, i32 0, i32 0), i32 %count)
  ret void
}

declare i32 @printf(i8*, ...)