pub const PROBLEM_LIST_QUERY: &str = r#"
query problemsetQuestionList($categorySlug: String, $limit: Int, $skip: Int, $filters: QuestionListFilterInput) {
  problemsetQuestionList: questionList(
    categorySlug: $categorySlug
    limit: $limit
    skip: $skip
    filters: $filters
  ) {
    total: totalNum
    questions: data {
      frontendQuestionId: questionFrontendId
      title
      titleSlug
      difficulty
      acRate
      isPaidOnly
      topicTags {
        name
        slug
      }
    }
  }
}
"#;

pub const QUESTION_DETAIL_QUERY: &str = r#"
query questionDetail($titleSlug: String!) {
  question(titleSlug: $titleSlug) {
    questionId
    frontendQuestionId: questionFrontendId
    title
    titleSlug
    difficulty
    content
    isPaidOnly
    topicTags {
      name
      slug
    }
    codeSnippets {
      lang
      langSlug
      code
    }
    hints
  }
}
"#;
