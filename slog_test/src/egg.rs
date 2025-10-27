#[cfg(test)]
mod tests {
   use slog::*;
   prelude!();

   #[test]
   fn points_to() {
      type Str = &'static str;
      
      local_db!(points_to_db, {
         (define eq: ascent_byods_rels::eqrel_canonical usize usize)
         (define classt Str)
         (define fieldt Str)

         // statements
         (define stmt usize)
         (define stmt_new Str usize)
         (define stmt_assign Str Str)
         (define stmt_store Str usize Str)
         (define stmt_load Str Str usize)

         // points to relations
         (define var_points_to Str usize)
         (define heap_points_to usize usize usize)

      }, {
         // typing rules
         (stmt ?(stmt_new _ (classt _)))
         (stmt ?(stmt_assign _ _))
         (stmt ?(stmt_store _ (fieldt _) _))
         (stmt ?(stmt_load _ _ (fieldt _)))
      }, slog_gen);

      points_to_db!(PointsToTest, {
         (define success i32)
         [(stmt_new a b) --> (var_points_to a b)]
         [(stmt_assign v1 v2) (var_points_to v2 c2) --> (var_points_to v1 c2)]
         [(stmt_load v1 v2 f) (var_points_to v2 c1) (heap_points_to c1 f c2) --> (var_points_to v1 c2)]
         [(stmt_store v1 f v2) (var_points_to v1 c1) (var_points_to v2 c2) --> (heap_points_to c1 f c2)]

         // EDB:
         (stmt_new "o1" (classt "A"))
         (stmt_new "o2" (classt "B"))
         (stmt_assign "o3" "o2")
         (stmt_store "o2" (fieldt "f") "o1")
         (stmt_load "r" "o3" (fieldt "f"))

         // queries
         [(success 1) <-- (var_points_to "o1" (classt "A"))]
         [(success 2) <-- (var_points_to "o2" (classt "B"))]
         [(success 3) <-- (var_points_to "o3" (classt "B"))]
         [(success 4) <-- (heap_points_to (classt "B") (fieldt "f") (classt "A"))]
         [(success 5) <-- (var_points_to "r" (classt "A"))]
      });

      let mut prog = PointsToTest::default();
      prog.run();
      println!("success: {:?}", prog.success);

   }
}

