use std::fmt::Write;

use placeholder_query_builder::{
    expr::{BinaryOp, Column, Expr, ExprId, Exprs},
    value::Value,
};

use crate::utils::JoinWrite;

use super::plan::{PgQuery, PgQueryPlan};

pub(crate) fn render_query(query: &PgQueryPlan) -> PgQuery {
    let mut params = Vec::new();
    let mut sql = String::new();

    JoinWrite {
        buf: &mut sql,
        items: query.select.iter().copied(),
        at_first: |sql| sql.write_str("SELECT "),
        r#do: |sql, id| {
            render_expr(sql, &query.exprs, id, &mut params);
            Ok(())
        },
        join: |sql| sql.write_str(", "),
        at_last: |sql| write!(sql, " FROM {} AS {}", query.from.name, query.from.alias),
    }
    .exec()
    .unwrap();

    for join in &query.joins {
        write!(sql, " JOIN {} AS {} ON ", join.table.name, join.table.alias).unwrap();
        render_expr(&mut sql, &query.exprs, join.on, &mut params);
    }

    JoinWrite {
        buf: &mut sql,
        items: query.filters.iter().copied(),
        at_first: |sql| sql.write_str(" WHERE "),
        r#do: |sql, id| {
            render_expr(sql, &query.exprs, id, &mut params);
            Ok(())
        },
        join: |sql| sql.write_str(" AND "),
        at_last: |_| Ok(()),
    }
    .exec()
    .unwrap();

    PgQuery { sql, params }
}

fn render_expr(sql: &mut String, exprs: &Exprs, id: ExprId, params: &mut Vec<Value>) {
    match exprs.get(id) {
        Expr::Column(column) => render_column(sql, column),
        Expr::Value(value) => {
            params.push(value.clone());
            write!(sql, "${}", params.len()).unwrap();
        }
        Expr::Values(values) => render_values(sql, values, params),
        Expr::Binary { op, left, right } => match op {
            BinaryOp::And => {
                sql.write_str("(").unwrap();
                render_expr(sql, exprs, *left, params);
                sql.write_str(" AND ").unwrap();
                render_expr(sql, exprs, *right, params);
                sql.write_str(")").unwrap();
            }
            BinaryOp::Or => {
                sql.write_str("(").unwrap();
                render_expr(sql, exprs, *left, params);
                sql.write_str(" OR ").unwrap();
                render_expr(sql, exprs, *right, params);
                sql.write_str(")").unwrap();
            }
            BinaryOp::Eq => {
                render_expr(sql, exprs, *left, params);
                sql.write_str(" = ").unwrap();
                render_expr(sql, exprs, *right, params);
            }
            BinaryOp::In => {
                render_expr(sql, exprs, *left, params);
                sql.write_str(" IN (").unwrap();
                render_expr(sql, exprs, *right, params);
                sql.write_str(")").unwrap();
            }
            BinaryOp::Like => {
                render_expr(sql, exprs, *left, params);
                sql.write_str(" LIKE ").unwrap();
                render_expr(sql, exprs, *right, params);
            }
        },
    }
}

fn render_values(sql: &mut String, values: &[Value], params: &mut Vec<Value>) {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            sql.write_str(", ").unwrap();
        }

        params.push(value.clone());
        write!(sql, "${}", params.len()).unwrap();
    }
}

fn render_column(sql: &mut String, column: &Column) {
    match column.schema() {
        Some(schema) => write!(sql, "{}.{}.{}", schema, column.table(), column.name()).unwrap(),
        None => write!(sql, "{}.{}", column.table(), column.name()).unwrap(),
    }
}
