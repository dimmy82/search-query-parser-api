use actix_web::{get, web, App, HttpServer, Responder};
use search_query_parser::{condition::Condition, condition::Operator, parse_query_to_condition};
use serde::Serialize;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(parse_query))
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
