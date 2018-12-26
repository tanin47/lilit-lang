; ModuleID = 'func_with_params.c'
source_filename = "func_with_params.c"
target datalayout = "e-m:e-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

%struct.I32 = type { i32 }
%struct.Number = type { %struct.I32* }

@formatString = private constant [10 x i8] c"hella %d  "
declare i32 @printf(i8*, ...)

define %struct.I32* @test(%struct.Number*, %struct.Number*) {
  %3 = alloca %struct.Number*
  %4 = alloca %struct.Number*
  store %struct.Number* %0, %struct.Number** %3
  store %struct.Number* %1, %struct.Number** %4
  %5 = load %struct.Number*, %struct.Number** %3
  %6 = getelementptr inbounds %struct.Number, %struct.Number* %5, i32 0, i32 0
  %7 = load %struct.I32*, %struct.I32** %6

  %i32_gep = getelementptr inbounds %struct.I32, %struct.I32* %7, i32 0, i32 0
  %d = load i32, i32* %i32_gep
  %call = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([10 x i8], [10 x i8]* @formatString , i32 0, i32 0), i32 %d)

  ret %struct.I32* %7
}

; Function Attrs: noinline nounwind optnone uwtable
define i32 @main() #0 {
  %1 = alloca i32
  %2 = alloca %struct.Number
  %3 = alloca %struct.I32
  %4 = alloca %struct.Number
  store i32 0, i32* %1
  %5 = getelementptr inbounds %struct.I32, %struct.I32* %3, i32 0, i32 0
  store i32 123, i32* %5
  %6 = getelementptr inbounds %struct.Number, %struct.Number* %2, i32 0, i32 0
  store %struct.I32* %3, %struct.I32** %6
  %7 = call %struct.I32* @test(%struct.Number* %2, %struct.Number* %4)
  %8 = getelementptr inbounds %struct.I32, %struct.I32* %7, i32 0, i32 0
  %9 = load i32, i32* %8
  ret i32 %9
}

attributes #0 = { noinline nounwind optnone uwtable "correctly-rounded-divide-sqrt-fp-math"="false" "disable-tail-calls"="false" "less-precise-fpmad"="false" "no-frame-pointer-elim"="true" "no-frame-pointer-elim-non-leaf" "no-infs-fp-math"="false" "no-jump-tables"="false" "no-nans-fp-math"="false" "no-signed-zeros-fp-math"="false" "no-trapping-math"="false" "stack-protector-buffer-size"="8" "target-cpu"="x86-64" "target-features"="+fxsr,+mmx,+sse,+sse2,+x87" "unsafe-fp-math"="false" "use-soft-float"="false" }

!llvm.module.flags = !{!0}
!llvm.ident = !{!1}

!0 = !{i32 1, !"wchar_size", i32 4}
!1 = !{!"clang version 6.0.1-svn334776-1~exp1~20181018153226.114 (branches/release_60)"}
