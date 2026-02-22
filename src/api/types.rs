use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
}

// Problem list types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemListData {
    pub problemset_question_list: Option<ProblemsetQuestionList>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemsetQuestionList {
    pub total: i32,
    pub questions: Vec<ProblemSummary>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemSummary {
    pub frontend_question_id: String,
    pub title: String,
    pub title_slug: String,
    pub difficulty: String,
    pub ac_rate: f64,
    pub is_paid_only: bool,
    pub topic_tags: Vec<TopicTag>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicTag {
    pub name: String,
    pub slug: String,
}

// Problem detail types
#[derive(Debug, Deserialize)]
pub struct QuestionDetailData {
    pub question: Option<QuestionDetail>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionDetail {
    pub question_id: String,
    pub frontend_question_id: String,
    pub title: String,
    pub title_slug: String,
    pub difficulty: String,
    pub content: Option<String>,
    pub is_paid_only: bool,
    pub topic_tags: Vec<TopicTag>,
    pub code_snippets: Option<Vec<CodeSnippet>>,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippet {
    pub lang: String,
    pub lang_slug: String,
    pub code: String,
}
