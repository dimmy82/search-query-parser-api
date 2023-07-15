# search-query-parser

## what is this application for

this is a REST Api application based on [search-query-parser](https://github.com/dimmy82/search-query-parser).

## usage

### 1. start application

```shell
$ ./fmt_check_test.sh
$ cargo run
```

### 2. parse query string to conditions via http request

```shell
# the query string before url encode is 「aaa and (-bbb or "ccc")」
$ curl 'http://localhost:12345/v1/query/aaa%20and%20(-bbb%20or%20%22ccc%22)'
```

the response will be look like

```json
{
  "and": [
    {
      "keyword": "aaa"
    },
    {
      "or": [
        {
          "not": {
            "keyword": "bbb"
          }
        },
        {
          "phraseKeyword": "ccc"
        }
      ]
    }
  ]
}
```

### 3. parse query string to elasticsearch dsl query via http request

```shell
# the query string before url encode is 「aaa and (-bbb or "ccc")」
$ curl 'http://localhost:12345/v1/query/aaa%20and%20(-bbb%20or%20%22ccc%22)/es_dsl'
```

the response will be look like

※ this is a elasticsearch dsl query sample, you should change the <target_field>

```json
{
  "bool": {
    "must": [
      {
        "match": {
          "target_field": "aaa"
        }
      },
      {
        "bool": {
          "should": [
            {
              "bool": {
                "must_not": {
                  "match": {
                    "target_field": "bbb"
                  }
                }
              }
            },
            {
              "match_phrase": {
                "target_field": "ccc"
              }
            }
          ]
        }
      }
    ]
  }
}
```
