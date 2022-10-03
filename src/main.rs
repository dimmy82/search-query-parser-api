use actix_web::{get, web, App, HttpServer, Responder};
use search_query_parser::{parse_query_to_condition, Condition, Operator};
use serde::Serialize;
use serde_json::{json, Value};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(parse_query)
            .service(parse_query_to_es_dsl)
    })
    .bind(("0.0.0.0", 12345))?
    .run()
    .await
}

#[get("/v1/query/{query_string}")]
async fn parse_query(query_string: web::Path<String>) -> impl Responder {
    match parse_query_to_condition(query_string.as_ref()) {
        Ok(condition) => web::Json(json(condition)),
        Err(e) => {
            println!("ERROR: {e}");
            web::Json(json(Condition::None))
        }
    }
}

#[get("/v1/query/{query_string}/es_dsl")]
async fn parse_query_to_es_dsl(query_string: web::Path<String>) -> impl Responder {
    match parse_query_to_condition(query_string.as_ref()) {
        Ok(condition) => web::Json(es_dsl_json(condition)),
        Err(e) => {
            println!("ERROR: {e}");
            web::Json(json!({}))
        }
    }
}

fn json(condition: Condition) -> ConditionJson {
    match condition {
        Condition::Keyword(value) => ConditionJson::keyword(value),
        Condition::PhraseKeyword(value) => ConditionJson::phrase_keyword(value),
        Condition::Not(value) => ConditionJson::not(json(*value)),
        Condition::Operator(operator, value) => match operator {
            Operator::And => ConditionJson::and(value.into_iter().map(|v| json(v)).collect()),
            Operator::Or => ConditionJson::or(value.into_iter().map(|v| json(v)).collect()),
        },
        _ => ConditionJson::none(),
    }
}

fn es_dsl_json(condition: Condition) -> Value {
    match condition {
        Condition::Keyword(value) => json!(
            {
                "match": {
                    "target_field": value
                }
            }
        ),
        Condition::PhraseKeyword(value) => json!(
            {
                "match_phrase": {
                    "target_field": value
                }
            }
        ),
        Condition::Not(value) => json!(
            {
                "bool": {
                    "must_not": es_dsl_json(*value)
                }
            }
        ),
        Condition::Operator(operator, value) => match operator {
            Operator::And => json!(
                {
                    "bool": {
                        "must": value.into_iter().map(|v| es_dsl_json(v)).collect::<Value>()
                    }
                }
            ),
            Operator::Or => json!(
                {
                    "bool": {
                        "should": value.into_iter().map(|v| es_dsl_json(v)).collect::<Value>()
                    }
                }
            ),
        },
        _ => json!({}),
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
struct ConditionJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    keyword: Option<String>,
    #[serde(rename(serialize = "phraseKeyword"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    phrase_keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    not: Option<Box<ConditionJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    and: Option<Vec<ConditionJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    or: Option<Vec<ConditionJson>>,
}

impl ConditionJson {
    fn none() -> Self {
        Self {
            keyword: None,
            phrase_keyword: None,
            not: None,
            and: None,
            or: None,
        }
    }
    fn keyword(value: String) -> Self {
        Self {
            keyword: Some(value),
            phrase_keyword: None,
            not: None,
            and: None,
            or: None,
        }
    }

    fn phrase_keyword(value: String) -> Self {
        Self {
            keyword: None,
            phrase_keyword: Some(value),
            not: None,
            and: None,
            or: None,
        }
    }

    fn not(value: ConditionJson) -> Self {
        Self {
            keyword: None,
            phrase_keyword: None,
            not: Some(Box::new(value)),
            and: None,
            or: None,
        }
    }

    fn and(value: Vec<ConditionJson>) -> Self {
        Self {
            keyword: None,
            phrase_keyword: None,
            not: None,
            and: Some(value),
            or: None,
        }
    }

    fn or(value: Vec<ConditionJson>) -> Self {
        Self {
            keyword: None,
            phrase_keyword: None,
            not: None,
            and: None,
            or: Some(value),
        }
    }
}
