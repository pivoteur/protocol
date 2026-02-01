use book::{
   err_utils::ErrStr,
   test_utils::{collate_results, mk_tests}
};

use quizzes::quiz03::a_partition::functional_tests::runoff_get_args;

#[tokio::main]
async fn main() -> ErrStr<()> {
   collate_results("a_partition", &mk_tests("partition open pivots",
      vec![runoff_get_args()]))
}

