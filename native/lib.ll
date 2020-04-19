; ModuleID = 'native/lib.c'
source_filename = "native/lib.c"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.Process = type { i64, i64, i64, i64 }
%struct.Test = type { i64, i64 }

@GC_finalizer_count = global i32 0, align 4
@.str = private unnamed_addr constant [20 x i8] c"GC freed count: %d\0A\00", align 1

; Function Attrs: noinline nounwind optnone uwtable
define %struct.Process* @lilit_exec(i8*) #0 {
  %2 = alloca i8*, align 8
  %3 = alloca i32*, align 8
  %4 = alloca i32*, align 8
  %5 = alloca i32*, align 8
  %6 = alloca i32, align 4
  %7 = alloca %struct.Process*, align 8
  store i8* %0, i8** %2, align 8
  %8 = call noalias i8* @GC_malloc(i64 8) #4
  %9 = bitcast i8* %8 to i32*
  store i32* %9, i32** %3, align 8
  %10 = call noalias i8* @GC_malloc(i64 8) #4
  %11 = bitcast i8* %10 to i32*
  store i32* %11, i32** %4, align 8
  %12 = call noalias i8* @GC_malloc(i64 8) #4
  %13 = bitcast i8* %12 to i32*
  store i32* %13, i32** %5, align 8
  %14 = load i32*, i32** %3, align 8
  %15 = call i32 @pipe(i32* %14) #5
  %16 = load i32*, i32** %4, align 8
  %17 = call i32 @pipe(i32* %16) #5
  %18 = load i32*, i32** %5, align 8
  %19 = call i32 @pipe(i32* %18) #5
  %20 = call i32 @fork() #5
  store i32 %20, i32* %6, align 4
  %21 = load i32, i32* %6, align 4
  %22 = icmp eq i32 %21, 0
  br i1 %22, label %23, label %51

; <label>:23:                                     ; preds = %1
  %24 = load i32*, i32** %3, align 8
  %25 = getelementptr inbounds i32, i32* %24, i64 1
  %26 = load i32, i32* %25, align 4
  %27 = call i32 @close(i32 %26)
  %28 = load i32*, i32** %4, align 8
  %29 = getelementptr inbounds i32, i32* %28, i64 0
  %30 = load i32, i32* %29, align 4
  %31 = call i32 @close(i32 %30)
  %32 = load i32*, i32** %5, align 8
  %33 = getelementptr inbounds i32, i32* %32, i64 0
  %34 = load i32, i32* %33, align 4
  %35 = call i32 @close(i32 %34)
  %36 = load i32*, i32** %3, align 8
  %37 = getelementptr inbounds i32, i32* %36, i64 0
  %38 = load i32, i32* %37, align 4
  %39 = call i32 @dup2(i32 %38, i32 0) #5
  %40 = load i32*, i32** %4, align 8
  %41 = getelementptr inbounds i32, i32* %40, i64 1
  %42 = load i32, i32* %41, align 4
  %43 = call i32 @dup2(i32 %42, i32 1) #5
  %44 = load i32*, i32** %5, align 8
  %45 = getelementptr inbounds i32, i32* %44, i64 1
  %46 = load i32, i32* %45, align 4
  %47 = call i32 @dup2(i32 %46, i32 2) #5
  %48 = load i8*, i8** %2, align 8
  %49 = load i8*, i8** %2, align 8
  %50 = call i32 (i8*, i8*, ...) @execlp(i8* %48, i8* %49, i8* null) #5
  br label %51

; <label>:51:                                     ; preds = %23, %1
  %52 = load i32*, i32** %3, align 8
  %53 = getelementptr inbounds i32, i32* %52, i64 0
  %54 = load i32, i32* %53, align 4
  %55 = call i32 @close(i32 %54)
  %56 = load i32*, i32** %4, align 8
  %57 = getelementptr inbounds i32, i32* %56, i64 1
  %58 = load i32, i32* %57, align 4
  %59 = call i32 @close(i32 %58)
  %60 = load i32*, i32** %5, align 8
  %61 = getelementptr inbounds i32, i32* %60, i64 1
  %62 = load i32, i32* %61, align 4
  %63 = call i32 @close(i32 %62)
  %64 = call noalias i8* @GC_malloc(i64 32) #4
  %65 = bitcast i8* %64 to %struct.Process*
  store %struct.Process* %65, %struct.Process** %7, align 8
  %66 = load i32, i32* %6, align 4
  %67 = sext i32 %66 to i64
  %68 = load %struct.Process*, %struct.Process** %7, align 8
  %69 = getelementptr inbounds %struct.Process, %struct.Process* %68, i32 0, i32 0
  store i64 %67, i64* %69, align 8
  %70 = load i32*, i32** %3, align 8
  %71 = getelementptr inbounds i32, i32* %70, i64 1
  %72 = load i32, i32* %71, align 4
  %73 = sext i32 %72 to i64
  %74 = load %struct.Process*, %struct.Process** %7, align 8
  %75 = getelementptr inbounds %struct.Process, %struct.Process* %74, i32 0, i32 1
  store i64 %73, i64* %75, align 8
  %76 = load i32*, i32** %4, align 8
  %77 = getelementptr inbounds i32, i32* %76, i64 0
  %78 = load i32, i32* %77, align 4
  %79 = sext i32 %78 to i64
  %80 = load %struct.Process*, %struct.Process** %7, align 8
  %81 = getelementptr inbounds %struct.Process, %struct.Process* %80, i32 0, i32 2
  store i64 %79, i64* %81, align 8
  %82 = load i32*, i32** %5, align 8
  %83 = getelementptr inbounds i32, i32* %82, i64 0
  %84 = load i32, i32* %83, align 4
  %85 = sext i32 %84 to i64
  %86 = load %struct.Process*, %struct.Process** %7, align 8
  %87 = getelementptr inbounds %struct.Process, %struct.Process* %86, i32 0, i32 3
  store i64 %85, i64* %87, align 8
  %88 = load %struct.Process*, %struct.Process** %7, align 8
  ret %struct.Process* %88
}

; Function Attrs: allocsize(0)
declare noalias i8* @GC_malloc(i64) #1

; Function Attrs: nounwind
declare i32 @pipe(i32*) #2

; Function Attrs: nounwind
declare i32 @fork() #2

declare i32 @close(i32) #3

; Function Attrs: nounwind
declare i32 @dup2(i32, i32) #2

; Function Attrs: nounwind
declare i32 @execlp(i8*, i8*, ...) #2

; Function Attrs: noinline nounwind optnone uwtable
define signext i8 @lilit_read(i32) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i8, align 1
  store i32 %0, i32* %2, align 4
  %4 = load i32, i32* %2, align 4
  %5 = call i64 @read(i32 %4, i8* %3, i64 1)
  %6 = load i8, i8* %3, align 1
  ret i8 %6
}

declare i64 @read(i32, i8*, i64) #3

; Function Attrs: noinline nounwind optnone uwtable
define void @lilit_write(i32, i8 signext) #0 {
  %3 = alloca i32, align 4
  %4 = alloca i8, align 1
  store i32 %0, i32* %3, align 4
  store i8 %1, i8* %4, align 1
  %5 = load i32, i32* %3, align 4
  %6 = call i64 @write(i32 %5, i8* %4, i64 1)
  ret void
}

declare i64 @write(i32, i8*, i64) #3

; Function Attrs: noinline nounwind optnone uwtable
define i32 @lilit_wait(i32) #0 {
  %2 = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 %0, i32* %2, align 4
  %4 = load i32, i32* %2, align 4
  %5 = call i32 @waitpid(i32 %4, i32* %3, i32 0)
  %6 = load i32, i32* %3, align 4
  ret i32 %6
}

declare i32 @waitpid(i32, i32*, i32) #3

; Function Attrs: noinline nounwind optnone uwtable
define %struct.Test* @test_call() #0 {
  %1 = alloca %struct.Test*, align 8
  %2 = call noalias i8* @GC_malloc(i64 16) #4
  %3 = bitcast i8* %2 to %struct.Test*
  store %struct.Test* %3, %struct.Test** %1, align 8
  %4 = load %struct.Test*, %struct.Test** %1, align 8
  %5 = getelementptr inbounds %struct.Test, %struct.Test* %4, i32 0, i32 0
  store i64 23, i64* %5, align 8
  %6 = load %struct.Test*, %struct.Test** %1, align 8
  %7 = getelementptr inbounds %struct.Test, %struct.Test* %6, i32 0, i32 1
  store i64 37, i64* %7, align 8
  %8 = load %struct.Test*, %struct.Test** %1, align 8
  ret %struct.Test* %8
}

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
  %8 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([20 x i8], [20 x i8]* @.str, i32 0, i32 0), i32 %7)
  ret void
}

declare i32 @printf(i8*, ...) #3

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #1 = { allocsize(0) "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #2 = { nounwind "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #3 = { "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }
attributes #4 = { allocsize(0) }
attributes #5 = { nounwind }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 6.0.1-svn334776-1~exp1~20181018153226.114 (branches/release_60)"}
