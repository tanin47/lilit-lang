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
