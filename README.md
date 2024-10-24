# magiceye

![](https://img.shields.io/badge/language-Rust-red) ![](https://img.shields.io/badge/version-0.1.0-brightgreen) [![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/myyrakle/magiceye/blob/master/LICENSE)

- table diff checker

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

- postgresql only

## Supported Report Language

- English
- Korean
