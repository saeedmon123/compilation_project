; ModuleID = 'program.ll'
source_filename = "miniimp"

define i64 @func(i64 %in) {
entry:
  br label %block4

block0:                                           ; preds = %block2, %block1
  %var1.0 = phi i64 [ %tmp2, %block1 ], [ %tmp4, %block2 ]
  ret i64 %var1.0

block1:                                           ; preds = %block3
  %tmp2 = add i64 %in, 8
  br label %block0

block2:                                           ; preds = %block3
  %tmp4 = sub i64 %in, 2
  br label %block0

block3:                                           ; preds = %block4
  %tmp6 = icmp slt i64 %in, 10
  br i1 %tmp6, label %block1, label %block2

block4:                                           ; preds = %entry
  br label %block3
}
