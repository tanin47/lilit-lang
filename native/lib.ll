%struct._IO_FILE = type opaque
declare i8* @fgets(i8*, i32, %struct._IO_FILE*)

@stdin = external global %struct._IO_FILE*, align 8

define i8* @read() {
  %1 = alloca [100 x i8]
  %2 = getelementptr inbounds [100 x i8], [100 x i8]* %1, i32 0, i32 0
  %3 = load %struct._IO_FILE*, %struct._IO_FILE** @stdin
  %4 = call i8* @fgets(i8* %2, i32 100, %struct._IO_FILE* %3)
  ret i8* %2
}

@finalizer.str.freed = private unnamed_addr constant [10 x i8] c"freed %d\0A\00"
@finalizer.count = internal global i32 0, align 4

define void @finalizer(i8*, i8*) {
  %count = load i32, i32* @finalizer.count, align 4
  %count2 = add nsw i32 %count, 1
  store i32 %count2, i32* @finalizer.count, align 4
  call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([10 x i8], [10 x i8]* @finalizer.str.freed, i32 0, i32 0), i32 %count)
  ret void
}

declare i32 @printf(i8*, ...)