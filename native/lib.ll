; ModuleID = 'native/lib.c'
source_filename = "native/lib.c"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct._IO_FILE = type { i32, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, i8*, %struct._IO_marker*, %struct._IO_FILE*, i32, i32, i64, i16, i8, [1 x i8], i8*, i64, i8*, i8*, i8*, i8*, i64, i32, [20 x i8] }
%struct._IO_marker = type { %struct._IO_marker*, %struct._IO_FILE*, i32 }

@stdin = external global %struct._IO_FILE*, align 8
@.str = private unnamed_addr constant [2 x i8] c"r\00", align 1
@GC_finalizer_count = global i32 0, align 4
@.str.1 = private unnamed_addr constant [20 x i8] c"GC freed count: %d\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define i8* @lilit__read() #0 {
  %1 = alloca i8*, align 8
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  %4 = alloca i8*, align 8
  %5 = alloca i8*, align 8
  %6 = alloca i32, align 4
  %7 = alloca i32, align 4
  store i32 3, i32* %2, align 4
  %8 = load i32, i32* %2, align 4
  store i32 %8, i32* %3, align 4
  %9 = load i32, i32* %3, align 4
  %10 = sext i32 %9 to i64
  %11 = mul i64 1, %10
  %12 = call noalias i8* @GC_malloc(i64 %11) #5
  store i8* %12, i8** %4, align 8
  %13 = load i8*, i8** %4, align 8
  store i8* %13, i8** %5, align 8
  br label %14

; <label>:14:                                     ; preds = %60, %0
  %15 = load i8*, i8** %5, align 8
  %16 = load i32, i32* %2, align 4
  %17 = load %struct._IO_FILE*, %struct._IO_FILE** @stdin, align 8
  %18 = call i8* @fgets(i8* %15, i32 %16, %struct._IO_FILE* %17)
  %19 = icmp ne i8* %18, null
  br i1 %19, label %20, label %61

; <label>:20:                                     ; preds = %14
  %21 = load i8*, i8** %5, align 8
  %22 = call i64 @strlen(i8* %21) #6
  %23 = trunc i64 %22 to i32
  store i32 %23, i32* %6, align 4
  %24 = load i32, i32* %6, align 4
  %25 = load i32, i32* %2, align 4
  %26 = sub nsw i32 %25, 1
  %27 = icmp eq i32 %24, %26
  br i1 %27, label %28, label %53

; <label>:28:                                     ; preds = %20
  %29 = load i8*, i8** %5, align 8
  %30 = load i32, i32* %6, align 4
  %31 = sub nsw i32 %30, 1
  %32 = sext i32 %31 to i64
  %33 = getelementptr inbounds i8, i8* %29, i64 %32
  %34 = load i8, i8* %33, align 1
  %35 = sext i8 %34 to i32
  %36 = icmp ne i32 %35, 10
  br i1 %36, label %37, label %53

; <label>:37:                                     ; preds = %28
  %38 = load i32, i32* %3, align 4
  %39 = load i32, i32* %2, align 4
  %40 = add nsw i32 %38, %39
  %41 = sub nsw i32 %40, 1
  store i32 %41, i32* %7, align 4
  %42 = load i8*, i8** %4, align 8
  %43 = load i32, i32* %7, align 4
  %44 = sext i32 %43 to i64
  %45 = mul i64 1, %44
  %46 = call i8* @GC_realloc(i8* %42, i64 %45) #7
  store i8* %46, i8** %4, align 8
  %47 = load i8*, i8** %4, align 8
  %48 = load i32, i32* %3, align 4
  %49 = sub nsw i32 %48, 1
  %50 = sext i32 %49 to i64
  %51 = getelementptr inbounds i8, i8* %47, i64 %50
  store i8* %51, i8** %5, align 8
  %52 = load i32, i32* %7, align 4
  store i32 %52, i32* %3, align 4
  br label %60

; <label>:53:                                     ; preds = %28, %20
  %54 = load i8*, i8** %5, align 8
  %55 = load i32, i32* %6, align 4
  %56 = sub nsw i32 %55, 1
  %57 = sext i32 %56 to i64
  %58 = getelementptr inbounds i8, i8* %54, i64 %57
  store i8 0, i8* %58, align 1
  %59 = load i8*, i8** %4, align 8
  store i8* %59, i8** %1, align 8
  br label %63

; <label>:60:                                     ; preds = %37
  br label %14

; <label>:61:                                     ; preds = %14
  %62 = load i8*, i8** %4, align 8
  store i8* %62, i8** %1, align 8
  br label %63

; <label>:63:                                     ; preds = %61, %53
  %64 = load i8*, i8** %1, align 8
  ret i8* %64
}

; Function Attrs: allocsize(0)
declare noalias i8* @GC_malloc(i64) #1

declare i8* @fgets(i8*, i32, %struct._IO_FILE*) #2

; Function Attrs: nounwind readonly
declare i64 @strlen(i8*) #3

; Function Attrs: allocsize(1)
declare i8* @GC_realloc(i8*, i64) #4

; Function Attrs: noinline nounwind optnone uwtable
define i8* @lilit__read_file(i8*) #0 {
  %2 = alloca i8*, align 8
  %3 = alloca %struct._IO_FILE*, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %6 = alloca i8*, align 8
  %7 = alloca i32, align 4
  %8 = alloca i8*, align 8
  %9 = alloca i32, align 4
  store i8* %0, i8** %2, align 8
  %10 = load i8*, i8** %2, align 8
  %11 = call %struct._IO_FILE* @fopen(i8* %10, i8* getelementptr inbounds ([2 x i8], [2 x i8]* @.str, i32 0, i32 0))
  store %struct._IO_FILE* %11, %struct._IO_FILE** %3, align 8
  store i32 11, i32* %4, align 4
  %12 = load i32, i32* %4, align 4
  store i32 %12, i32* %5, align 4
  %13 = load i32, i32* %5, align 4
  %14 = sext i32 %13 to i64
  %15 = mul i64 1, %14
  %16 = call noalias i8* @GC_malloc(i64 %15) #5
  store i8* %16, i8** %6, align 8
  store i32 0, i32* %7, align 4
  %17 = load i8*, i8** %6, align 8
  store i8* %17, i8** %8, align 8
  br label %18

; <label>:18:                                     ; preds = %26, %1
  %19 = load i8*, i8** %8, align 8
  %20 = load i32, i32* %4, align 4
  %21 = sext i32 %20 to i64
  %22 = load %struct._IO_FILE*, %struct._IO_FILE** %3, align 8
  %23 = call i64 @fread(i8* %19, i64 1, i64 %21, %struct._IO_FILE* %22)
  %24 = trunc i64 %23 to i32
  store i32 %24, i32* %7, align 4
  %25 = icmp sgt i32 %24, 0
  br i1 %25, label %26, label %40

; <label>:26:                                     ; preds = %18
  %27 = load i32, i32* %5, align 4
  %28 = load i32, i32* %4, align 4
  %29 = add nsw i32 %27, %28
  store i32 %29, i32* %9, align 4
  %30 = load i8*, i8** %6, align 8
  %31 = load i32, i32* %9, align 4
  %32 = sext i32 %31 to i64
  %33 = mul i64 1, %32
  %34 = call i8* @GC_realloc(i8* %30, i64 %33) #7
  store i8* %34, i8** %6, align 8
  %35 = load i8*, i8** %6, align 8
  %36 = load i32, i32* %5, align 4
  %37 = sext i32 %36 to i64
  %38 = getelementptr inbounds i8, i8* %35, i64 %37
  store i8* %38, i8** %8, align 8
  %39 = load i32, i32* %9, align 4
  store i32 %39, i32* %5, align 4
  br label %18

; <label>:40:                                     ; preds = %18
  %41 = load i8*, i8** %6, align 8
  ret i8* %41
}

declare %struct._IO_FILE* @fopen(i8*, i8*) #2

declare i64 @fread(i8*, i64, i64, %struct._IO_FILE*) #2

; Function Attrs: noinline nounwind optnone uwtable
define void @GC_finalizer(i8*, i8*) #0 {
  %3 = alloca i8*, align 8
  %4 = alloca i8*, align 8
  store i8* %0, i8** %3, align 8
  store i8* %1, i8** %4, align 8
  %5 = load i32, i32* @GC_finalizer_count, align 4
  %6 = add nsw i32 %5, 1
  store i32 %6, i32* @GC_finalizer_count, align 4
  %7 = load i32, i32* @GC_finalizer_count, align 4
  %8 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([20 x i8], [20 x i8]* @.str.1, i32 0, i32 0), i32 %7)
  ret void
}

declare i32 @printf(i8*, ...) #2

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { allocsize(0) "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #2 = { "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #3 = { nounwind readonly "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #4 = { allocsize(1) "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #5 = { allocsize(0) }
attributes #6 = { nounwind readonly }
attributes #7 = { allocsize(1) }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 6.0.1-svn334776-1~exp1~20181018153226.114 (branches/release_60)"}
