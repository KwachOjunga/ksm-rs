%string = type { i64, i8* }
@str.0 = constant [1 x i8] c"\00"
@str.1 = constant [10 x i8] c"fib = %d\0A\00"

declare i8* @malloc(i64)

declare i8* @realloc(i8*, i64)

declare i8* @memcpy(i8* %dest, i8* %src, i64 %n)

declare i8* @strcat(i8*, i8*)

declare i8* @strcpy(i8*, i8*)

declare i8* @strncpy(i8*, i8*, i64)

declare i8* @strndup(i8*, i64)

declare void @exit(i32)

define i64 @string_len(%string %input) {
entry:
        %0 = extractvalue %string %input, 0
        ret i64 %0
}
