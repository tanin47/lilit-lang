%Boolean = type { %"@Boolean"* }
%"@Boolean" = type { i1 }
%"@I32" = type { i32 }
%Number = type { %"@I32"* }

%struct._IO_FILE = type opaque
declare i8* @fgets(i8*, i32, %struct._IO_FILE*)

@stdin = external global %struct._IO_FILE*, align 8

define i8* @read() {
  %1 = alloca [100 x i8]
  %2 = getelementptr inbounds [100 x i8], [100 x i8]* %1, i32 0, i32 0
  %3 = load %struct._IO_FILE*, %struct._IO_FILE** null
  %4 = call i8* @fgets(i8* %2, i32 100, %struct._IO_FILE* %3)
  ret i8* null
}

declare noalias i8* @GC_malloc(i64)

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

declare void @GC_register_finalizer(i8*, void (i8*, i8*)*, i8*, void (i8*, i8*)**, i8**)

define i32 @main() {
first_block:
  call void @GC_init()
  %malloc = call i8* @GC_malloc(i64 ptrtoint (i1** getelementptr (i1*, i1** null, i32 1) to i64))
  call void @GC_register_finalizer(i8* %malloc, void (i8*, i8*)* @finalizer, i8* null, void (i8*, i8*)** null, i8** null)
  %cast = bitcast i8* %malloc to %Number*
  %malloc1 = call i8* @GC_malloc(i64 ptrtoint (i32* getelementptr (i32, i32* null, i32 1) to i64))
  call void @GC_register_finalizer(i8* %malloc1, void (i8*, i8*)* @finalizer, i8* null, void (i8*, i8*)** null, i8** null)
  %cast2 = bitcast i8* %malloc1 to %"@I32"*
  %"gep for member" = getelementptr inbounds %"@I32", %"@I32"* %cast2, i32 0, i32 0
  store i32 1, i32* %"gep for member"
  %gep = getelementptr inbounds %Number, %Number* %cast, i32 0, i32 0
  store %"@I32"* %cast2, %"@I32"** %gep
  call void @GC_gcollect()
  %"gep for @I32" = getelementptr inbounds %Number, %Number* %cast, i32 0, i32 0
  %"load @I32" = load %"@I32"*, %"@I32"** %"gep for @I32"
  %"gep for the first param of @I32" = getelementptr inbounds %"@I32", %"@I32"* %"load @I32", i32 0, i32 0
  %"load the first param of @I32" = load i32, i32* %"gep for the first param of @I32"
  ret i32 %"load the first param of @I32"
}


declare void @GC_init()
declare void @GC_gcollect()
