use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

use super::queries::{PROBLEM_LIST_QUERY, QUESTION_DETAIL_QUERY};
use super::types::*;

const LEETCODE_GRAPHQL: &str = "https://leetcode.com/graphql";

#[derive(Clone)]
pub struct LeetCodeClient {
    client: Client,
}

impl LeetCodeClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .context("Failed to create HTTP client")?;
        Ok(Self { client })
    }

    pub async fn fetch_problems(
        &self,
        limit: i32,
        skip: i32,
        difficulty: Option<&str>,
    ) -> Result<(Vec<ProblemSummary>, i32)> {
        let mut filters = json!({});
        if let Some(diff) = difficulty {
            filters = json!({ "difficulty": diff });
        }

        let body = json!({
            "query": PROBLEM_LIST_QUERY,
            "variables": {
                "categorySlug": "all-code-essentials",
                "limit": limit,
                "skip": skip,
                "filters": filters,
            }
        });

        let resp = self
            .client
            .post(LEETCODE_GRAPHQL)
            .header("Content-Type", "application/json")
            .header("Referer", "https://leetcode.com/problemset/")
            .json(&body)
            .send()
            .await
            .context("Failed to send problem list request")?;

        let data: GraphQLResponse<ProblemListData> = resp
            .json()
            .await
            .context("Failed to parse problem list response")?;

        let list = data
            .data
            .and_then(|d| d.problemset_question_list)
            .context("No problem list data in response")?;

        Ok((list.questions, list.total))
    }

    pub async fn fetch_problem_detail(&self, slug: &str) -> Result<QuestionDetail> {
        let body = json!({
            "query": QUESTION_DETAIL_QUERY,
            "variables": {
                "titleSlug": slug,
            }
        });

        let resp = self
            .client
            .post(LEETCODE_GRAPHQL)
            .header("Content-Type", "application/json")
            .header("Referer", format!("https://leetcode.com/problems/{}/", slug))
            .json(&body)
            .send()
            .await
            .context("Failed to send problem detail request")?;

        let data: GraphQLResponse<QuestionDetailData> = resp
            .json()
            .await
            .context("Failed to parse problem detail response")?;

        data.data
            .and_then(|d| d.question)
            .context("No question data in response")
    }
}
