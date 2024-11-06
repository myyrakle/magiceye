# magiceye

![](https://img.shields.io/badge/language-Rust-red) ![](https://img.shields.io/badge/version-0.2.0-brightgreen) [![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/myyrakle/magiceye/blob/master/LICENSE)

- database diff checker

## What is this?

- When managing a DB with multiple versions such as production, stage, qa, develop, etc., a situation may arise where the DB schema is partially inconsistent.
- In such situations, this tool detects and reports inconsistencies between databases.

## Get Started

Install using cargo

```bash
cargo install magiceye
```

Then, use the init command to enter database information, etc.

```bash
magiceye init
```

This is the database where the base connection URL is the reference point.

magiceye detects and reports the following two targets:

1. Something in the base database but not in the target database.
2. It exists in both the base database and the target database, but the types are different.

Once you have completed the settings through the init command, you can start collecting reports with the run command.

```bash
magiceye run
```

If you have a lot of tables, collecting DDL information may take some time.

When processing is complete, a report file is created in the form "2024-01-30 18:53.json".

## Supported DBMS

- postgresql
- mysql

## Supported Report Language

- English
- Korean

## Report Example 

```json
{
  "report_table_list": [
    {
      "table_name": "followers",
      "report_list": [
        "Index: followers.idx_follower_follower_id exists in the base database, but not in the target database."
      ]
    },
    {
      "table_name": "reports_fk_test",
      "report_list": [
        "Index: reports_fk_test.post_id exists in the base database, but not in the target database.",
        "Foreign Key: reports_fk_test.reports_fk_test_ibfk_2 exists in the base database, but not in the target database."
      ]
    },
    {
      "table_name": "posts",
      "report_list": [
        "Column: posts.id has different AUTO_INCREMENT. => AUTO_INCREMENT != NOT AUTO_INCREMENT",
        "Column: posts.title has different default value. => asdf != "
      ]
    },
    {
      "table_name": "tags",
      "report_list": [
        "Column: tags.name has different data type. => varchar(255) != varchar(155)"
      ]
    }
  ]
}
```